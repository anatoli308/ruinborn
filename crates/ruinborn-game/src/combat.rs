//! Enemy AI + combat + loot drops.
//!
//! Phase 1: simple Idle -> Chase -> Attack -> Dead state machine. Linear chase,
//! no pathfinding. GOAP can replace this engine in Phase 2 by swapping out
//! `tick_enemy` and feeding the same Enemy struct.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::damage::{DamageInstance, DamageTag, DamageType, DotInstance, Resistances};
use crate::items::{roll_random_item, Item};
use crate::world::{zone_by_id, Zone, ZoneId};

pub const MELEE_RANGE: f64 = 2.5;
pub const AGGRO_RANGE: f64 = 12.0;
pub const LEASH_RANGE: f64 = 25.0;
pub const ENEMY_DESPAWN_TICKS: u32 = 30;
pub const LOOT_DROP_CHANCE: f64 = 0.45;
pub const LOOT_PICKUP_RANGE: f64 = 2.5;
pub const PLAYER_ATTACK_RANGE: f64 = 4.0;

/// Visible enemy archetype. Drives stats + visuals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyKind {
    Zombie,
    Skeleton,
    FallenOne,
}

impl EnemyKind {
    pub fn label(self) -> &'static str {
        match self {
            EnemyKind::Zombie => "Zombie",
            EnemyKind::Skeleton => "Skelett",
            EnemyKind::FallenOne => "Fallener",
        }
    }

    /// Per-archetype damage type for the basic melee swing.
    pub fn attack_damage_type(self) -> DamageType {
        match self {
            // Zombies bite — physical, but inject poison via `poison_on_hit`.
            EnemyKind::Zombie => DamageType::Physical,
            EnemyKind::Skeleton => DamageType::Physical,
            // Fallen ones are little fire imps in our flavor.
            EnemyKind::FallenOne => DamageType::Fire,
        }
    }

    /// Optional poison rider on melee hit: (dps, ticks).
    pub fn poison_on_hit(self) -> Option<(f64, u32)> {
        match self {
            EnemyKind::Zombie => Some((1.5, 10)),
            _ => None,
        }
    }

    /// Per-archetype resistance profile.
    pub fn resistances(self) -> Resistances {
        match self {
            // Rotting flesh — shrugs off poison, weak to fire.
            EnemyKind::Zombie => Resistances {
                physical: 0.0,
                fire: -25.0,
                cold: 10.0,
                lightning: 0.0,
                poison: 75.0,
                magical: 0.0,
            },
            // Bones — laughs at piercing (physical) and cold, hates blunt magic.
            EnemyKind::Skeleton => Resistances {
                physical: 25.0,
                fire: 0.0,
                cold: 50.0,
                lightning: -25.0,
                poison: 100.0,
                magical: 0.0,
            },
            // Fire imps — fire-immune-ish, cold-vulnerable.
            EnemyKind::FallenOne => Resistances {
                physical: 0.0,
                fire: 75.0,
                cold: -50.0,
                lightning: 0.0,
                poison: 0.0,
                magical: 25.0,
            },
        }
    }
    /// (max_hp, damage, move_speed_per_tick, xp_reward, gold_drop_min, gold_drop_max)
    pub fn base_stats(self, level: u32) -> EnemyBaseStats {
        let l = level as f64;
        match self {
            EnemyKind::Zombie => EnemyBaseStats {
                max_hp: 25.0 + 8.0 * l,
                damage: 4.0 + 1.5 * l,
                move_speed: 0.10,
                xp_reward: (15.0 + 6.0 * l) as u64,
                gold_min: (2.0 + l) as u32,
                gold_max: (8.0 + 3.0 * l) as u32,
            },
            EnemyKind::Skeleton => EnemyBaseStats {
                max_hp: 18.0 + 6.0 * l,
                damage: 5.0 + 1.8 * l,
                move_speed: 0.14,
                xp_reward: (18.0 + 7.0 * l) as u64,
                gold_min: (3.0 + l) as u32,
                gold_max: (10.0 + 3.0 * l) as u32,
            },
            EnemyKind::FallenOne => EnemyBaseStats {
                max_hp: 12.0 + 5.0 * l,
                damage: 3.0 + 1.0 * l,
                move_speed: 0.18,
                xp_reward: (10.0 + 5.0 * l) as u64,
                gold_min: (1.0 + l) as u32,
                gold_max: (6.0 + 2.0 * l) as u32,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnemyBaseStats {
    pub max_hp: f64,
    pub damage: f64,
    pub move_speed: f64,
    pub xp_reward: u64,
    pub gold_min: u32,
    pub gold_max: u32,
}

/// Discrete AI state the enemy is currently in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyState {
    Idle,
    Chase,
    Attack,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub id: String,
    pub kind: EnemyKind,
    pub zone: ZoneId,
    pub x: f64,
    pub z: f64,
    pub hp: f64,
    pub max_hp: f64,
    pub damage: f64,
    pub level: u32,
    pub move_speed: f64,
    pub xp_reward: u64,
    pub state: EnemyState,
    pub target_player_id: Option<String>,
    /// Ticks remaining on attack cooldown (0 = ready).
    pub attack_cooldown: u32,
    /// Ticks remaining until cleanup if dead.
    pub despawn_in: u32,
    /// Spawn anchor — used for leashing.
    pub spawn_x: f64,
    pub spawn_z: f64,
    /// Per-type damage reduction.
    #[serde(default)]
    pub resistances: Resistances,
    /// Active damage-over-time effects.
    #[serde(default)]
    pub dots: Vec<DotInstance>,
}

impl Enemy {
    pub fn is_alive(&self) -> bool {
        self.state != EnemyState::Dead && self.hp > 0.0
    }
}

/// Loot lying on the ground after an enemy died.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootDrop {
    pub id: String,
    pub item: Item,
    pub x: f64,
    pub z: f64,
    pub zone: ZoneId,
    pub dropped_tick: u64,
}

/// Spawn a single enemy somewhere inside a zone.
pub fn spawn_enemy(
    next_id: &mut u64,
    zone: &Zone,
    rng: &mut impl Rng,
) -> Option<Enemy> {
    if zone.enemy_target == 0 {
        return None;
    }
    // Random kind, weighted by zone difficulty (dungeon -> tougher).
    let kind = match zone.kind {
        crate::world::ZoneKind::Wilderness => {
            if rng.gen_bool(0.6) { EnemyKind::Zombie } else { EnemyKind::FallenOne }
        }
        crate::world::ZoneKind::Dungeon => {
            if rng.gen_bool(0.5) { EnemyKind::Skeleton } else { EnemyKind::Zombie }
        }
        crate::world::ZoneKind::Town => return None,
    };
    let level = match zone.kind {
        crate::world::ZoneKind::Wilderness => rng.gen_range(1..=3),
        crate::world::ZoneKind::Dungeon => rng.gen_range(3..=6),
        crate::world::ZoneKind::Town => 1,
    };
    let stats = kind.base_stats(level);
    let pad = 4.0;
    let x = rng.gen_range((zone.bounds.min_x + pad)..(zone.bounds.max_x - pad));
    let z = rng.gen_range((zone.bounds.min_z + pad)..(zone.bounds.max_z - pad));
    *next_id += 1;
    Some(Enemy {
        id: format!("enemy_{}", *next_id),
        kind,
        zone: zone.id,
        x,
        z,
        hp: stats.max_hp,
        max_hp: stats.max_hp,
        damage: stats.damage,
        level,
        move_speed: stats.move_speed,
        xp_reward: stats.xp_reward,
        state: EnemyState::Idle,
        target_player_id: None,
        attack_cooldown: 0,
        despawn_in: 0,
        spawn_x: x,
        spawn_z: z,
        resistances: kind.resistances(),
        dots: Vec::new(),
    })
}

/// Maintain enemy population per zone.
pub fn maintain_population(
    enemies: &mut Vec<Enemy>,
    zones: &[Zone],
    next_id: &mut u64,
    rng: &mut impl Rng,
) {
    // Drop dead-too-long.
    enemies.retain(|e| !(e.state == EnemyState::Dead && e.despawn_in == 0));

    for zone in zones {
        if zone.enemy_target == 0 {
            continue;
        }
        let alive = enemies.iter()
            .filter(|e| e.zone == zone.id && e.is_alive())
            .count() as u32;
        if alive < zone.enemy_target {
            // Spawn 1 per maintenance pass to avoid bursty spawns.
            if let Some(en) = spawn_enemy(next_id, zone, rng) {
                enemies.push(en);
            }
        }
    }
}

/// Hit produced by a player attack against an enemy.
#[derive(Debug, Clone)]
pub struct PlayerAttackOutcome {
    pub damage_dealt: f64,
    pub damage_type: DamageType,
    pub killed: bool,
    pub xp_reward: u64,
    pub gold_reward: u32,
    pub loot: Option<Item>,
    pub enemy_label: String,
}

/// One enemy melee hit on a player. Carries typed damage + optional DoT rider.
#[derive(Debug, Clone)]
pub struct EnemyHit {
    pub player_id: String,
    pub damage: DamageInstance,
    pub poison_dot: Option<DotInstance>,
}

/// Apply damage to an enemy; if dead, roll loot. Returns `None` if not in range / not found.
pub fn player_attack_enemy(
    enemies: &mut [Enemy],
    enemy_id: &str,
    player_x: f64,
    player_z: f64,
    base_damage: f64,
    current_tick: u64,
    rng: &mut impl Rng,
) -> Option<PlayerAttackOutcome> {
    let enemy = enemies.iter_mut().find(|e| e.id == enemy_id)?;
    if !enemy.is_alive() {
        return None;
    }
    let dx = enemy.x - player_x;
    let dz = enemy.z - player_z;
    let dist = (dx * dx + dz * dz).sqrt();
    if dist > PLAYER_ATTACK_RANGE {
        return None;
    }
    let dmg = DamageInstance::physical_melee(base_damage);
    let actual = enemy.resistances.apply(&dmg);
    enemy.hp -= actual;
    let _ = current_tick;
    let mut outcome = PlayerAttackOutcome {
        damage_dealt: actual,
        damage_type: DamageType::Physical,
        killed: false,
        xp_reward: 0,
        gold_reward: 0,
        loot: None,
        enemy_label: enemy.kind.label().to_string(),
    };
    if enemy.hp <= 0.0 {
        enemy.hp = 0.0;
        enemy.state = EnemyState::Dead;
        enemy.despawn_in = ENEMY_DESPAWN_TICKS;
        outcome.killed = true;
        outcome.xp_reward = enemy.xp_reward;
        let stats = enemy.kind.base_stats(enemy.level);
        outcome.gold_reward = if stats.gold_max > stats.gold_min {
            rng.gen_range(stats.gold_min..=stats.gold_max)
        } else {
            stats.gold_min
        };
        if rng.gen_bool(LOOT_DROP_CHANCE) {
            outcome.loot = Some(roll_random_item(current_tick, rng));
        }
    }
    Some(outcome)
}

/// Apply a typed [`DamageInstance`] to an enemy without any range check (used by skills,
/// which enforce their own per-skill range). Honours per-type resistances. Handles
/// death + loot rolls just like `player_attack_enemy`.
pub fn deal_damage_to_enemy(
    enemies: &mut [Enemy],
    enemy_id: &str,
    damage: DamageInstance,
    current_tick: u64,
    rng: &mut impl Rng,
) -> Option<PlayerAttackOutcome> {
    let enemy = enemies.iter_mut().find(|e| e.id == enemy_id)?;
    if !enemy.is_alive() {
        return None;
    }
    let actual = enemy.resistances.apply(&damage);
    enemy.hp -= actual;
    let _ = current_tick;
    let mut outcome = PlayerAttackOutcome {
        damage_dealt: actual,
        damage_type: damage.damage_type,
        killed: false,
        xp_reward: 0,
        gold_reward: 0,
        loot: None,
        enemy_label: enemy.kind.label().to_string(),
    };
    if enemy.hp <= 0.0 {
        enemy.hp = 0.0;
        enemy.state = EnemyState::Dead;
        enemy.despawn_in = ENEMY_DESPAWN_TICKS;
        outcome.killed = true;
        outcome.xp_reward = enemy.xp_reward;
        let stats = enemy.kind.base_stats(enemy.level);
        outcome.gold_reward = if stats.gold_max > stats.gold_min {
            rng.gen_range(stats.gold_min..=stats.gold_max)
        } else {
            stats.gold_min
        };
        if rng.gen_bool(LOOT_DROP_CHANCE) {
            outcome.loot = Some(roll_random_item(current_tick, rng));
        }
    }
    Some(outcome)
}

/// Apply a [`DotInstance`] to an enemy. Stacks by replacing the longer-lasting one.
pub fn apply_dot_to_enemy(enemies: &mut [Enemy], enemy_id: &str, dot: DotInstance) {
    if let Some(enemy) = enemies.iter_mut().find(|e| e.id == enemy_id) {
        if !enemy.is_alive() { return; }
        // Replace existing same-type DoT if the new one is at least as strong.
        if let Some(existing) = enemy.dots.iter_mut().find(|d| d.damage_type == dot.damage_type) {
            if dot.damage_per_tick * dot.ticks_remaining as f64
                >= existing.damage_per_tick * existing.ticks_remaining as f64
            {
                *existing = dot;
            }
        } else {
            enemy.dots.push(dot);
        }
    }
}

/// Per-tick AI. Reads player positions, mutates enemies. Returns typed damage
/// events with optional DoT riders so the caller can apply them to player HP +
/// dot list.
pub fn tick_enemies(
    enemies: &mut [Enemy],
    zones: &[Zone],
    player_positions: &HashMap<String, (ZoneId, f64, f64, bool)>,
) -> Vec<EnemyHit> {
    let mut hits: Vec<EnemyHit> = Vec::new();

    for enemy in enemies.iter_mut() {
        match enemy.state {
            EnemyState::Dead => {
                if enemy.despawn_in > 0 {
                    enemy.despawn_in -= 1;
                }
                continue;
            }
            _ => {}
        }
        if enemy.attack_cooldown > 0 {
            enemy.attack_cooldown -= 1;
        }

        // Find closest live player in same zone.
        let mut best: Option<(String, f64, f64, f64)> = None;
        for (pid, (pzone, px, pz, alive)) in player_positions.iter() {
            if !*alive { continue; }
            if *pzone != enemy.zone { continue; }
            let dx = px - enemy.x;
            let dz = pz - enemy.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist > AGGRO_RANGE { continue; }
            match &best {
                Some((_, _, _, d)) if *d <= dist => {}
                _ => best = Some((pid.clone(), *px, *pz, dist)),
            }
        }

        // Leash check: if too far from spawn, return.
        let leash = ((enemy.x - enemy.spawn_x).powi(2) + (enemy.z - enemy.spawn_z).powi(2)).sqrt();
        if leash > LEASH_RANGE {
            // Move toward spawn.
            let dx = enemy.spawn_x - enemy.x;
            let dz = enemy.spawn_z - enemy.z;
            let n = (dx * dx + dz * dz).sqrt().max(0.0001);
            enemy.x += dx / n * enemy.move_speed;
            enemy.z += dz / n * enemy.move_speed;
            enemy.state = EnemyState::Idle;
            enemy.target_player_id = None;
            continue;
        }

        match best {
            None => {
                enemy.state = EnemyState::Idle;
                enemy.target_player_id = None;
            }
            Some((pid, px, pz, dist)) => {
                enemy.target_player_id = Some(pid.clone());
                if dist <= MELEE_RANGE {
                    enemy.state = EnemyState::Attack;
                    if enemy.attack_cooldown == 0 {
                        let dmg = DamageInstance::new(
                            enemy.kind.attack_damage_type(),
                            enemy.damage,
                            &[DamageTag::Melee],
                        );
                        let poison_dot = enemy.kind.poison_on_hit()
                            .map(|(dps, ticks)| DotInstance {
                                damage_type: DamageType::Poison,
                                damage_per_tick: dps,
                                ticks_remaining: ticks,
                                tags: vec![DamageTag::Melee],
                            });
                        hits.push(EnemyHit { player_id: pid, damage: dmg, poison_dot });
                        enemy.attack_cooldown = 8; // ~1.5s at 5tps
                    }
                } else {
                    enemy.state = EnemyState::Chase;
                    let dx = px - enemy.x;
                    let dz = pz - enemy.z;
                    let n = (dx * dx + dz * dz).sqrt().max(0.0001);
                    enemy.x += dx / n * enemy.move_speed;
                    enemy.z += dz / n * enemy.move_speed;
                    // Stay inside zone bounds.
                    if let Some(zone) = zone_by_id(zones, enemy.zone) {
                        let (cx, cz) = zone.bounds.clamp(enemy.x, enemy.z);
                        enemy.x = cx;
                        enemy.z = cz;
                    }
                }
            }
        }
    }

    hits
}
