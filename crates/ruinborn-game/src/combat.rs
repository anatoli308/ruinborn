//! Enemy AI + combat + loot drops.
//!
//! Phase 1: simple Idle -> Chase -> Attack -> Dead state machine. Linear chase,
//! no pathfinding. GOAP can replace this engine in Phase 2 by swapping out
//! `tick_enemy` and feeding the same Enemy struct.
//!
//! Stats are JSON-driven — see [`crate::enemy_archetype`]. `Enemy.kind` is a
//! `String` archetype id; all per-monster numbers are looked up via
//! [`enemy_archetype::archetype`].

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ai::boids::{self, BoidParams, BoidSample};
use crate::ai::goap::{
    self, GoapAgentRuntime, RuntimeStep, SenseValue,
};
use crate::damage::{DamageInstance, DamageTag, DamageType, DotInstance, Resistances};
use crate::enemy_archetype::{self, try_archetype, AiKind, EnemyArchetype};
use crate::items::{roll_random_item, Item};
use crate::world::{zone_by_id, Zone, ZoneId};

// All time-denominated combat values are in **combat ticks** (20 Hz = 50 ms each).
// Distances are in world units (unchanged).
pub const MELEE_RANGE: f64 = 2.5;
pub const AGGRO_RANGE: f64 = 12.0;
pub const LEASH_RANGE: f64 = 25.0;
/// Corpse linger before despawn (~30 s @ 20 Hz).
pub const ENEMY_DESPAWN_TICKS: u32 = 600;
pub const LOOT_DROP_CHANCE: f64 = 0.45;
pub const LOOT_PICKUP_RANGE: f64 = 2.5;
pub const PLAYER_ATTACK_RANGE: f64 = 4.0;

/// Stable archetype id (e.g. `"zombie"`, `"skeleton"`). The legal set is
/// defined by `data/enemies.json`; see [`enemy_archetype`].
pub type EnemyKind = String;

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
    /// GOAP runtime state for archetypes whose `ai` is `goap`.
    /// Server-only — not part of the wire snapshot to keep client
    /// payloads small.
    #[serde(default, skip_serializing, skip_deserializing)]
    pub goap: Option<GoapAgentRuntime>,
    /// Movement delta produced by the *previous* tick. Used by the
    /// boids alignment rule as a heading proxy. Server-only.
    #[serde(default, skip_serializing, skip_deserializing)]
    pub last_vx: f64,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub last_vz: f64,
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
fn build_enemy_at(
    next_id: &mut u64,
    zone_id: &ZoneId,
    arch: &EnemyArchetype,
    level: u32,
    x: f64,
    z: f64,
) -> Enemy {
    *next_id += 1;
    // Initialise a GOAP runtime if (a) the archetype opted in and
    // (b) a matching agent config was registered. If either is
    // missing we silently fall back to the legacy simple-chase AI.
    let goap = if arch.ai == AiKind::Goap {
        goap::try_agent_config(&arch.id).map(GoapAgentRuntime::new)
    } else {
        None
    };
    Enemy {
        id: format!("enemy_{}", *next_id),
        kind: arch.id.clone(),
        zone: zone_id.clone(),
        x,
        z,
        hp: arch.max_hp(level),
        max_hp: arch.max_hp(level),
        damage: arch.damage(level),
        level,
        move_speed: arch.move_speed,
        xp_reward: arch.xp_reward(level),
        state: EnemyState::Idle,
        target_player_id: None,
        attack_cooldown: 0,
        last_vx: 0.0,
        last_vz: 0.0,
        despawn_in: 0,
        spawn_x: x,
        spawn_z: z,
        resistances: arch.resistances,
        dots: Vec::new(),
        goap,
    }
}

/// Spawn a single random enemy in `zone`. Used as a fallback when packs aren't desired.
pub fn spawn_enemy(
    next_id: &mut u64,
    zone: &Zone,
    rng: &mut impl Rng,
) -> Option<Enemy> {
    if zone.enemy_target == 0 {
        return None;
    }
    let (arch, level) = enemy_archetype::pick_archetype_for_zone(zone.kind, rng)?;
    let pad = 4.0;
    let x = rng.gen_range((zone.bounds.min_x + pad)..(zone.bounds.max_x - pad));
    let z = rng.gen_range((zone.bounds.min_z + pad)..(zone.bounds.max_z - pad));
    Some(build_enemy_at(next_id, &zone.id, arch, level, x, z))
}

/// Spawn a tight cluster ("pack") of enemies in `zone`. The anchor is rolled
/// inside the zone bounds and members are placed within `pack_radius` of it.
/// Returns the list of spawned enemies (could be fewer than requested if the
/// archetype has no spawn rules for that zone).
pub fn spawn_pack(
    next_id: &mut u64,
    zone: &Zone,
    rng: &mut impl Rng,
) -> Vec<Enemy> {
    if zone.enemy_target == 0 {
        return Vec::new();
    }
    let Some((arch, level)) = enemy_archetype::pick_archetype_for_zone(zone.kind, rng) else {
        return Vec::new();
    };
    let pad = 4.0;
    let anchor_x =
        rng.gen_range((zone.bounds.min_x + pad)..(zone.bounds.max_x - pad));
    let anchor_z =
        rng.gen_range((zone.bounds.min_z + pad)..(zone.bounds.max_z - pad));

    let size = arch.pack_size(rng) as usize;
    let radius = arch.pack_radius.max(0.5);
    let mut out = Vec::with_capacity(size);
    for _ in 0..size {
        // Uniform disc sample around the anchor.
        let theta: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
        let r: f64 = radius * rng.gen::<f64>().sqrt();
        let mut x = anchor_x + theta.cos() * r;
        let mut z = anchor_z + theta.sin() * r;
        // Clamp inside zone bounds with the same padding.
        x = x.clamp(zone.bounds.min_x + pad, zone.bounds.max_x - pad);
        z = z.clamp(zone.bounds.min_z + pad, zone.bounds.max_z - pad);
        out.push(build_enemy_at(next_id, &zone.id, arch, level, x, z));
    }
    out
}

/// Maintain enemy population per zone. Spawns whole packs at a time so the
/// world feels populated in clumps (D2 cold plains-style) instead of a
/// uniform sprinkle.
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
            // Spawn a single pack per maintenance pass to avoid bursty spawns;
            // the population will fill up over a few economy ticks.
            let pack = spawn_pack(next_id, zone, rng);
            for member in pack {
                if (enemies.iter().filter(|e| e.zone == zone.id && e.is_alive()).count() as u32)
                    >= zone.enemy_target
                {
                    break;
                }
                enemies.push(member);
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
    let arch = try_archetype(&enemy.kind);
    let label = arch.map(|a| a.label.clone()).unwrap_or_else(|| enemy.kind.clone());
    let mut outcome = PlayerAttackOutcome {
        damage_dealt: actual,
        damage_type: DamageType::Physical,
        killed: false,
        xp_reward: 0,
        gold_reward: 0,
        loot: None,
        enemy_label: label,
    };
    if enemy.hp <= 0.0 {
        enemy.hp = 0.0;
        enemy.state = EnemyState::Dead;
        enemy.despawn_in = ENEMY_DESPAWN_TICKS;
        outcome.killed = true;
        outcome.xp_reward = enemy.xp_reward;
        if let Some(a) = arch {
            let gmin = a.gold_min(enemy.level);
            let gmax = a.gold_max(enemy.level);
            outcome.gold_reward = if gmax > gmin { rng.gen_range(gmin..=gmax) } else { gmin };
        }
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
    let arch = try_archetype(&enemy.kind);
    let label = arch.map(|a| a.label.clone()).unwrap_or_else(|| enemy.kind.clone());
    let mut outcome = PlayerAttackOutcome {
        damage_dealt: actual,
        damage_type: damage.damage_type,
        killed: false,
        xp_reward: 0,
        gold_reward: 0,
        loot: None,
        enemy_label: label,
    };
    if enemy.hp <= 0.0 {
        enemy.hp = 0.0;
        enemy.state = EnemyState::Dead;
        enemy.despawn_in = ENEMY_DESPAWN_TICKS;
        outcome.killed = true;
        outcome.xp_reward = enemy.xp_reward;
        if let Some(a) = arch {
            let gmin = a.gold_min(enemy.level);
            let gmax = a.gold_max(enemy.level);
            outcome.gold_reward = if gmax > gmin { rng.gen_range(gmin..=gmax) } else { gmin };
        }
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

    // Snapshot positions + previous-tick velocities for the boids
    // steering pass. Built once before the mutation loop so each
    // enemy can sample its same-kind neighbours without contending
    // for the &mut borrow.
    let snapshot = build_boid_snapshot(enemies);
    let boid_params = BoidParams::default();

    for i in 0..enemies.len() {
        let enemy = &mut enemies[i];

        match enemy.state {
            EnemyState::Dead => {
                if enemy.despawn_in > 0 {
                    enemy.despawn_in -= 1;
                }
                enemy.last_vx = 0.0;
                enemy.last_vz = 0.0;
                continue;
            }
            _ => {}
        }
        if enemy.attack_cooldown > 0 {
            enemy.attack_cooldown -= 1;
        }

        // Find closest live player in same zone — needed by both AI paths.
        let best = nearest_player_in_aggro(enemy, player_positions);

        // Leash check: if too far from spawn, return — overrides AI.
        let leash = ((enemy.x - enemy.spawn_x).powi(2) + (enemy.z - enemy.spawn_z).powi(2)).sqrt();
        if leash > LEASH_RANGE {
            let dx = enemy.spawn_x - enemy.x;
            let dz = enemy.spawn_z - enemy.z;
            let n = (dx * dx + dz * dz).sqrt().max(0.0001);
            let step_x = dx / n * enemy.move_speed;
            let step_z = dz / n * enemy.move_speed;
            enemy.x += step_x;
            enemy.z += step_z;
            enemy.last_vx = step_x;
            enemy.last_vz = step_z;
            enemy.state = EnemyState::Idle;
            enemy.target_player_id = None;
            // Drop GOAP plan — sensors will reseed on next tick.
            if let Some(rt) = enemy.goap.as_mut() {
                rt.force_replan();
            }
            continue;
        }

        if enemy.goap.is_some() {
            tick_enemy_goap(enemy, zones, best.as_ref(), &mut hits, i, &snapshot, &boid_params);
        } else {
            tick_enemy_simple(enemy, zones, best.as_ref(), &mut hits);
        }
    }

    hits
}

/// Build a per-enemy snapshot of the data the boids steering layer
/// reads. Keeps the inner loop branch-free and side-effect-free.
fn build_boid_snapshot(enemies: &[Enemy]) -> Vec<BoidSample> {
    use std::collections::HashMap;
    let mut kind_ids: HashMap<&str, u32> = HashMap::new();
    let mut next_id: u32 = 0;
    enemies
        .iter()
        .map(|e| {
            let slot = *kind_ids.entry(e.kind.as_str()).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            BoidSample {
                x: e.x,
                z: e.z,
                vx: e.last_vx,
                vz: e.last_vz,
                zone: e.zone.clone(),
                kind_slot: slot,
                alive: e.is_alive(),
            }
        })
        .collect()
}

/// Closest live player inside aggro range that shares the enemy's zone.
fn nearest_player_in_aggro(
    enemy: &Enemy,
    player_positions: &HashMap<String, (ZoneId, f64, f64, bool)>,
) -> Option<(String, f64, f64, f64)> {
    let mut best: Option<(String, f64, f64, f64)> = None;
    for (pid, (pzone, px, pz, alive)) in player_positions.iter() {
        if !*alive {
            continue;
        }
        if pzone != &enemy.zone {
            continue;
        }
        let dx = px - enemy.x;
        let dz = pz - enemy.z;
        let dist = (dx * dx + dz * dz).sqrt();
        if dist > AGGRO_RANGE {
            continue;
        }
        match &best {
            Some((_, _, _, d)) if *d <= dist => {}
            _ => best = Some((pid.clone(), *px, *pz, dist)),
        }
    }
    best
}

/// Legacy hard-coded AI: Idle → Chase → Attack on the closest player.
fn tick_enemy_simple(
    enemy: &mut Enemy,
    zones: &[Zone],
    best: Option<&(String, f64, f64, f64)>,
    hits: &mut Vec<EnemyHit>,
) {
    match best {
        None => {
            enemy.state = EnemyState::Idle;
            enemy.target_player_id = None;
            enemy.last_vx = 0.0;
            enemy.last_vz = 0.0;
        }
        Some((pid, px, pz, dist)) => {
            enemy.target_player_id = Some(pid.clone());
            if *dist <= MELEE_RANGE {
                enemy.state = EnemyState::Attack;
                enemy.last_vx = 0.0;
                enemy.last_vz = 0.0;
                if enemy.attack_cooldown == 0 {
                    let arch = try_archetype(&enemy.kind);
                    let dmg_type = arch
                        .map(|a| a.attack_damage_type)
                        .unwrap_or(DamageType::Physical);
                    let cooldown = arch.map(|a| a.attack_cooldown_ticks).unwrap_or(30);
                    let dmg = DamageInstance::new(dmg_type, enemy.damage, &[DamageTag::Melee]);
                    let poison_dot = arch.and_then(|a| a.poison_on_hit).map(|p| DotInstance {
                        damage_type: DamageType::Poison,
                        damage_per_tick: p.damage_per_tick,
                        ticks_remaining: p.ticks,
                        tags: vec![DamageTag::Melee],
                    });
                    hits.push(EnemyHit {
                        player_id: pid.clone(),
                        damage: dmg,
                        poison_dot,
                    });
                    enemy.attack_cooldown = cooldown;
                }
            } else {
                enemy.state = EnemyState::Chase;
                let dx = px - enemy.x;
                let dz = pz - enemy.z;
                let n = (dx * dx + dz * dz).sqrt().max(0.0001);
                let step_x = dx / n * enemy.move_speed;
                let step_z = dz / n * enemy.move_speed;
                enemy.x += step_x;
                enemy.z += step_z;
                if let Some(zone) = zone_by_id(zones, &enemy.zone) {
                    let (cx, cz) = zone.bounds.clamp(enemy.x, enemy.z);
                    enemy.x = cx;
                    enemy.z = cz;
                }
                enemy.last_vx = step_x;
                enemy.last_vz = step_z;
            }
        }
    }
}

/// GOAP-driven AI: senses → planner tick → behaviour dispatch.
///
/// Behaviour tags supported (must match those in `data/goap/agents.json`):
///
/// - `acquire_nearest_player` — copies the best-target id onto the enemy.
/// - `move_to_target` — chase clamped to zone bounds, with boids
///   steering (separation + alignment + cohesion) mixed into the
///   desired direction so packs spread out instead of stacking.
/// - `melee_attack` — emits a hit on the *completing* tick of the action;
///   carries archetype-defined damage type + optional poison rider.
/// - `wander` — no-op idle; reserved for future random-walk behaviour.
fn tick_enemy_goap(
    enemy: &mut Enemy,
    zones: &[Zone],
    best: Option<&(String, f64, f64, f64)>,
    hits: &mut Vec<EnemyHit>,
    self_idx: usize,
    snapshot: &[BoidSample],
    boid_params: &BoidParams,
) {
    // Resolve the agent config by archetype id. If it disappears
    // mid-run (hot reload edge case), fall back to legacy AI for
    // this tick.
    let Some(agent) = goap::try_agent_config(&enemy.kind) else {
        tick_enemy_simple(enemy, zones, best, hits);
        return;
    };

    // ----- 1. Sensor pass --------------------------------------------------
    write_goap_sensors(enemy, best);

    // ----- 2. Planner tick -------------------------------------------------
    // Borrow split: take &mut goap, agent is &'static.
    let runtime = enemy.goap.as_mut().expect("goap.is_some() checked by caller");
    let step = runtime.tick(agent);

    // ----- 3. Behaviour dispatch ------------------------------------------
    match step {
        RuntimeStep::Idle => {
            enemy.state = EnemyState::Idle;
            enemy.target_player_id = None;
            enemy.last_vx = 0.0;
            enemy.last_vz = 0.0;
        }
        RuntimeStep::Running {
            action_id,
            completing,
            ..
        } => {
            // Look up the action's behaviour tag.
            let behaviour = agent
                .action(&action_id)
                .map(|a| a.behaviour.as_str())
                .unwrap_or("");

            match behaviour {
                "acquire_nearest_player" => {
                    enemy.state = EnemyState::Idle;
                    enemy.target_player_id = best.map(|b| b.0.clone());
                    enemy.last_vx = 0.0;
                    enemy.last_vz = 0.0;
                }
                "move_to_target" => {
                    enemy.state = EnemyState::Chase;
                    if let Some((pid, px, pz, _)) = best {
                        enemy.target_player_id = Some(pid.clone());
                        // Desired heading toward target (unit vector).
                        let dx = px - enemy.x;
                        let dz = pz - enemy.z;
                        let n = (dx * dx + dz * dz).sqrt().max(0.0001);
                        let mut dir_x = dx / n;
                        let mut dir_z = dz / n;

                        // Mix in flocking steering. The offset is
                        // already magnitude-bounded, so adding it
                        // cannot dominate the pursuit direction.
                        let (sx, sz) = boids::flocking_offset(self_idx, snapshot, boid_params);
                        dir_x += sx;
                        dir_z += sz;

                        // Renormalise so we never exceed `move_speed`.
                        let m = (dir_x * dir_x + dir_z * dir_z).sqrt().max(0.0001);
                        let step_x = dir_x / m * enemy.move_speed;
                        let step_z = dir_z / m * enemy.move_speed;

                        enemy.x += step_x;
                        enemy.z += step_z;
                        if let Some(zone) = zone_by_id(zones, &enemy.zone) {
                            let (cx, cz) = zone.bounds.clamp(enemy.x, enemy.z);
                            enemy.x = cx;
                            enemy.z = cz;
                        }
                        enemy.last_vx = step_x;
                        enemy.last_vz = step_z;
                    } else {
                        enemy.last_vx = 0.0;
                        enemy.last_vz = 0.0;
                    }
                }
                "melee_attack" => {
                    enemy.state = EnemyState::Attack;
                    enemy.last_vx = 0.0;
                    enemy.last_vz = 0.0;
                    // Hit lands only on the *completing* tick — that
                    // gives us the natural attack cadence baked into
                    // `ticks_to_perform`. No reliance on
                    // `attack_cooldown` here.
                    if completing {
                        if let Some((pid, _, _, dist)) = best {
                            if *dist <= MELEE_RANGE {
                                let arch = try_archetype(&enemy.kind);
                                let dmg_type = arch
                                    .map(|a| a.attack_damage_type)
                                    .unwrap_or(DamageType::Physical);
                                let dmg = DamageInstance::new(
                                    dmg_type,
                                    enemy.damage,
                                    &[DamageTag::Melee],
                                );
                                let poison_dot =
                                    arch.and_then(|a| a.poison_on_hit).map(|p| DotInstance {
                                        damage_type: DamageType::Poison,
                                        damage_per_tick: p.damage_per_tick,
                                        ticks_remaining: p.ticks,
                                        tags: vec![DamageTag::Melee],
                                    });
                                hits.push(EnemyHit {
                                    player_id: pid.clone(),
                                    damage: dmg,
                                    poison_dot,
                                });
                            }
                        }
                    }
                }
                "wander" => {
                    enemy.state = EnemyState::Idle;
                    enemy.target_player_id = None;
                    enemy.last_vx = 0.0;
                    enemy.last_vz = 0.0;
                }
                _ => {
                    // Unknown behaviour — keep the enemy idle and fail
                    // safe rather than panicking. Authoring bug surfaces
                    // as a stationary monster, not a server crash.
                    enemy.state = EnemyState::Idle;
                    enemy.last_vx = 0.0;
                    enemy.last_vz = 0.0;
                }
            }

            // Apply effects + advance the plan on the completing tick.
            if completing {
                runtime.complete_action(agent);
            }
        }
    }
}

/// Write the world-state keys our authored agents read each tick.
/// Keep this in sync with `data/goap/agents.json`.
fn write_goap_sensors(enemy: &mut Enemy, best: Option<&(String, f64, f64, f64)>) {
    let runtime = enemy
        .goap
        .as_mut()
        .expect("write_goap_sensors called without runtime");

    let has_target = best.is_some();
    let in_attack_range = best.map(|b| b.3 <= MELEE_RANGE).unwrap_or(false);
    // We never sense `target_dead` from world state — that fact is
    // produced exclusively by the `melee_attack` effect. The plan
    // completes, the runtime clears it, and the next sensor pass
    // re-evaluates from scratch.

    runtime.set_world("has_target", SenseValue::Bool(has_target));
    runtime.set_world("in_attack_range", SenseValue::Bool(in_attack_range));
    runtime.set_world("target_dead", SenseValue::Bool(false));
}
