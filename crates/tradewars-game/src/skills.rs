//! Class-specific active skills — D2-flavoured, simple Phase 2 implementation.
//!
//! Phase 2 shape:
//! - Each skill has a static [`SkillDef`] (mana cost, cooldown, base damage, range, level req).
//! - The player's `allocated_skills` map tracks per-skill levels (0..=20). Class
//!   starter skills are always considered learned even at allocated level 0.
//! - Cast effects are kept intentionally simple: direct damage, AOE around the
//!   caster, self-buff, debuff, or teleport. Phase 3 (damage model) will rewire
//!   these to emit `DamageInstance`s with type/tags.

use rand::Rng;
use std::collections::HashMap;

use crate::classes::{class_definition, ClassId};
use crate::combat::{apply_dot_to_enemy, deal_damage_to_enemy};
use crate::damage::{DamageInstance, DamageTag, DamageType, DotInstance};
use crate::market::{ActionResult, GameState, PlayerState};
use crate::progression::grant_xp;

/// What the cast does mechanically. Phase 3: typed damage + tags + DoTs.
#[derive(Debug, Clone)]
pub enum SkillEffect {
    /// Direct damage to a single targeted enemy.
    DirectDamage { min: f64, max: f64 },
    /// AOE damage around the caster's current position.
    AoeAround { radius: f64, min: f64, max: f64 },
    /// Apply a damage-over-time on a single targeted enemy.
    DamageOverTime { dps: f64, ticks: u32 },
    /// Teleport caster up to `range` units toward the target point.
    Teleport,
    /// Self-buff with id stored in `active_buffs[buff_id] = duration_ticks`.
    SelfBuff { buff_id: &'static str, duration_ticks: u32 },
    /// Placeholder — not implemented yet.
    Placeholder,
}

/// Static metadata about an active skill.
#[derive(Debug, Clone)]
pub struct SkillDef {
    pub id: &'static str,
    pub name: &'static str,
    pub class_id: ClassId,
    pub mana_cost: f64,
    pub cooldown_ticks: u32,
    pub range: f64,
    pub requires_level: u32,
    pub effect: SkillEffect,
    /// Damage type for DamageInstance creation. `None` for non-damage skills.
    pub damage_type: Option<DamageType>,
    /// Tags applied to every DamageInstance this skill emits.
    pub tags: &'static [DamageTag],
}

/// Build the static catalog. Cheap enough to call on demand.
pub fn skill_catalog() -> Vec<SkillDef> {
    use ClassId::*;
    use DamageTag::*;
    use SkillEffect::*;

    vec![
        // ── Barbarian ──
        SkillDef {
            id: "bash",
            name: "Bash",
            class_id: Barbarian,
            mana_cost: 3.0,
            cooldown_ticks: 3,
            range: 3.0,
            requires_level: 1,
            effect: DirectDamage { min: 6.0, max: 12.0 },
            damage_type: Some(DamageType::Physical),
            tags: &[Melee],
        },
        SkillDef {
            id: "cleave",
            name: "Cleave",
            class_id: Barbarian,
            mana_cost: 6.0,
            cooldown_ticks: 8,
            range: 3.0,
            requires_level: 6,
            effect: AoeAround { radius: 3.5, min: 4.0, max: 9.0 },
            damage_type: Some(DamageType::Physical),
            tags: &[Melee],
        },
        SkillDef {
            id: "battle_cry",
            name: "Schlachtruf",
            class_id: Barbarian,
            mana_cost: 10.0,
            cooldown_ticks: 50,
            range: 0.0,
            requires_level: 12,
            effect: SelfBuff { buff_id: "battle_cry", duration_ticks: 25 },
            damage_type: None,
            tags: &[],
        },
        // ── Sorceress ──
        SkillDef {
            id: "fireball",
            name: "Feuerball",
            class_id: Sorceress,
            mana_cost: 5.0,
            cooldown_ticks: 5,
            range: 12.0,
            requires_level: 1,
            effect: DirectDamage { min: 10.0, max: 18.0 },
            damage_type: Some(DamageType::Fire),
            tags: &[Spell, Ranged],
        },
        SkillDef {
            id: "frost_nova",
            name: "Frostnova",
            class_id: Sorceress,
            mana_cost: 9.0,
            cooldown_ticks: 25,
            range: 0.0,
            requires_level: 6,
            effect: AoeAround { radius: 5.0, min: 5.0, max: 10.0 },
            damage_type: Some(DamageType::Cold),
            tags: &[Spell],
        },
        SkillDef {
            id: "teleport",
            name: "Teleport",
            class_id: Sorceress,
            mana_cost: 12.0,
            cooldown_ticks: 10,
            range: 15.0,
            requires_level: 12,
            effect: Teleport,
            damage_type: None,
            tags: &[Spell],
        },
        // ── Necromancer ──
        SkillDef {
            id: "bone_spear",
            name: "Knochenspeer",
            class_id: Necromancer,
            mana_cost: 4.0,
            cooldown_ticks: 4,
            range: 14.0,
            requires_level: 1,
            effect: DirectDamage { min: 8.0, max: 14.0 },
            damage_type: Some(DamageType::Magical),
            tags: &[Spell, Ranged],
        },
        SkillDef {
            id: "raise_skeleton",
            name: "Skelett beschwören",
            class_id: Necromancer,
            mana_cost: 15.0,
            cooldown_ticks: 30,
            range: 0.0,
            requires_level: 6,
            effect: Placeholder,
            damage_type: None,
            tags: &[Summoning],
        },
        SkillDef {
            id: "amplify_damage",
            name: "Giftwolke",
            class_id: Necromancer,
            mana_cost: 6.0,
            cooldown_ticks: 15,
            range: 12.0,
            requires_level: 12,
            effect: DamageOverTime { dps: 4.0, ticks: 20 },
            damage_type: Some(DamageType::Poison),
            tags: &[Spell, Trap],
        },
    ]
}

/// O(n) lookup. Catalog is small (~9 entries) so this is fine.
pub fn skill_def(id: &str) -> Option<SkillDef> {
    skill_catalog().into_iter().find(|s| s.id == id)
}

/// All skills available to a class.
pub fn skills_for_class(class: ClassId) -> Vec<SkillDef> {
    skill_catalog().into_iter().filter(|s| s.class_id == class).collect()
}

/// True if the player has the skill in their kit (starter or allocated >= 1).
pub fn player_knows_skill(player: &PlayerState, skill_id: &str) -> bool {
    let Some(class) = player.class_id else { return false; };
    if class_definition(class).starter_skills.iter().any(|s| s == skill_id) {
        return true;
    }
    player.allocated_skills.get(skill_id).copied().unwrap_or(0) >= 1
}

/// Spend one unspent skill point on a skill the player meets level requirement for.
pub fn allocate_skill(state: &mut GameState, player_id: &str, skill_id: &str) -> ActionResult {
    let Some(player) = state.players.get_mut(player_id) else {
        return ActionResult { success: false, message: "Spieler nicht gefunden".into() };
    };
    let Some(class) = player.class_id else {
        return ActionResult { success: false, message: "Wähle zuerst eine Klasse.".into() };
    };
    let Some(def) = skill_def(skill_id) else {
        return ActionResult { success: false, message: "Unbekannte Fertigkeit.".into() };
    };
    if def.class_id != class {
        return ActionResult { success: false, message: "Falsche Klasse für diese Fertigkeit.".into() };
    }
    if player.level < def.requires_level {
        return ActionResult {
            success: false,
            message: format!("Benötigt Stufe {}.", def.requires_level),
        };
    }
    if player.unspent_skill_points == 0 {
        return ActionResult { success: false, message: "Keine freien Fertigkeitspunkte.".into() };
    }
    let entry = player.allocated_skills.entry(skill_id.to_string()).or_insert(0);
    *entry += 1;
    let new_level = *entry;
    player.unspent_skill_points -= 1;
    player.notification = format!("✨ {} (Stufe {})", def.name, new_level);
    ActionResult { success: true, message: "OK".into() }
}

/// Cast an active skill. `target_*` are optional and only used by certain effects.
pub fn cast_skill(
    state: &mut GameState,
    player_id: &str,
    skill_id: &str,
    target_enemy_id: Option<&str>,
    target_x: Option<f64>,
    target_z: Option<f64>,
) -> ActionResult {
    let mut rng = rand::thread_rng();

    // Phase 1: validate caster & skill, deduct mana & set cooldown.
    let (px, pz, base_bonus, def) = {
        let Some(p) = state.players.get_mut(player_id) else {
            return ActionResult { success: false, message: "Spieler nicht gefunden".into() };
        };
        if p.is_dead {
            return ActionResult { success: false, message: "Du bist tot.".into() };
        }
        if p.class_id.is_none() {
            return ActionResult { success: false, message: "Wähle zuerst eine Klasse.".into() };
        }
        let Some(def) = skill_def(skill_id) else {
            return ActionResult { success: false, message: "Unbekannte Fertigkeit.".into() };
        };
        if !player_knows_skill(p, skill_id) {
            return ActionResult { success: false, message: "Du beherrschst diese Fertigkeit nicht.".into() };
        }
        if let Some(&cd) = p.skill_cooldowns.get(skill_id) {
            if cd > 0 {
                return ActionResult { success: false, message: "Fertigkeit lädt noch nach.".into() };
            }
        }
        if p.mana < def.mana_cost {
            return ActionResult { success: false, message: "Nicht genug Mana.".into() };
        }
        p.mana -= def.mana_cost;
        if def.cooldown_ticks > 0 {
            p.skill_cooldowns.insert(skill_id.to_string(), def.cooldown_ticks);
        }
        // Strength bonus applies to damage-dealing skills (Phase 3 will refine via DamageTag).
        let bonus = p.stats.melee_bonus() * 0.5;
        (p.x, p.z, bonus, def)
    };

    // Phase 2: apply effect.
    let mut killed_loot: Vec<(String, crate::items::Item, ZoneCoords)> = Vec::new();
    let mut total_xp: u64 = 0;
    let mut total_gold: u32 = 0;
    let mut killed_label: Option<String> = None;

    match &def.effect {
        SkillEffect::DirectDamage { min, max } => {
            let Some(enemy_id) = target_enemy_id else {
                return ActionResult { success: false, message: "Kein Ziel.".into() };
            };
            // Range check from current position.
            let in_range = state.enemies.iter().find(|e| e.id == enemy_id)
                .map(|e| ((e.x - px).powi(2) + (e.z - pz).powi(2)).sqrt() <= def.range)
                .unwrap_or(false);
            if !in_range {
                return ActionResult { success: false, message: "Ziel außer Reichweite.".into() };
            }
            let amount = rng.gen_range(*min..=*max) + base_bonus;
            let dtype = def.damage_type.unwrap_or(DamageType::Physical);
            let dmg = DamageInstance::new(dtype, amount, def.tags);
            let outcome = deal_damage_to_enemy(&mut state.enemies, enemy_id, dmg, state.tick, &mut rng);
            if let Some(o) = outcome {
                if o.killed {
                    if let Some(en) = state.enemies.iter().find(|e| e.id == enemy_id) {
                        if let Some(item) = o.loot.clone() {
                            killed_loot.push((en.id.clone(), item, ZoneCoords { x: en.x, z: en.z, zone: en.zone }));
                        }
                    }
                    total_xp += o.xp_reward;
                    total_gold += o.gold_reward;
                    killed_label = Some(o.enemy_label);
                }
            }
        }
        SkillEffect::AoeAround { radius, min, max } => {
            let r2 = radius * radius;
            let target_ids: Vec<String> = state.enemies.iter()
                .filter(|e| e.is_alive() && (e.x - px).powi(2) + (e.z - pz).powi(2) <= r2)
                .map(|e| e.id.clone())
                .collect();
            for eid in &target_ids {
                let amount = rng.gen_range(*min..=*max) + base_bonus * 0.5;
                let dtype = def.damage_type.unwrap_or(DamageType::Physical);
                let dmg = DamageInstance::new(dtype, amount, def.tags);
                if let Some(o) = deal_damage_to_enemy(&mut state.enemies, eid, dmg, state.tick, &mut rng) {
                    if o.killed {
                        if let Some(en) = state.enemies.iter().find(|e| &e.id == eid) {
                            if let Some(item) = o.loot.clone() {
                                killed_loot.push((en.id.clone(), item, ZoneCoords { x: en.x, z: en.z, zone: en.zone }));
                            }
                        }
                        total_xp += o.xp_reward;
                        total_gold += o.gold_reward;
                        killed_label = Some(o.enemy_label);
                    }
                }
            }
        }
        SkillEffect::DamageOverTime { dps, ticks } => {
            let Some(enemy_id) = target_enemy_id else {
                return ActionResult { success: false, message: "Kein Ziel.".into() };
            };
            let in_range = state.enemies.iter().find(|e| e.id == enemy_id)
                .map(|e| ((e.x - px).powi(2) + (e.z - pz).powi(2)).sqrt() <= def.range)
                .unwrap_or(false);
            if !in_range {
                return ActionResult { success: false, message: "Ziel außer Reichweite.".into() };
            }
            let dtype = def.damage_type.unwrap_or(DamageType::Poison);
            let dot = DotInstance {
                damage_type: dtype,
                damage_per_tick: *dps,
                ticks_remaining: *ticks,
                tags: def.tags.to_vec(),
            };
            apply_dot_to_enemy(&mut state.enemies, enemy_id, dot);
        }
        SkillEffect::Teleport => {
            let (Some(tx), Some(tz)) = (target_x, target_z) else {
                return ActionResult { success: false, message: "Kein Zielpunkt.".into() };
            };
            let dx = tx - px;
            let dz = tz - pz;
            let dist = (dx * dx + dz * dz).sqrt();
            let (nx, nz) = if dist > def.range && dist > 0.0001 {
                (px + dx / dist * def.range, pz + dz / dist * def.range)
            } else {
                (tx, tz)
            };
            if let Some(p) = state.players.get_mut(player_id) {
                p.x = nx;
                p.z = nz;
            }
        }
        SkillEffect::SelfBuff { buff_id, duration_ticks } => {
            if let Some(p) = state.players.get_mut(player_id) {
                p.active_buffs.insert((*buff_id).to_string(), *duration_ticks);
            }
        }
        SkillEffect::Placeholder => {
            // No mechanical effect yet — Phase 3.
        }
    }

    // Spawn loot drops + grant XP/gold.
    for (_eid, item, coords) in killed_loot {
        state.next_loot_id += 1;
        state.loot_drops.push(crate::combat::LootDrop {
            id: format!("loot_{}", state.next_loot_id),
            item,
            x: coords.x,
            z: coords.z,
            zone: coords.zone,
            dropped_tick: state.tick,
        });
    }
    if total_xp > 0 || total_gold > 0 {
        if let Some(p) = state.players.get_mut(player_id) {
            p.gold += total_gold as f64;
            let stats_clone = p.stats.clone();
            let levels = grant_xp(
                &mut p.level, &mut p.xp, &mut p.xp_to_next, &mut p.unspent_stat_points,
                &mut p.hp, &mut p.max_hp, &mut p.mana, &mut p.max_mana,
                &stats_clone, total_xp,
            );
            // Skill points: 1 per level after level 1.
            p.unspent_skill_points = p.unspent_skill_points.saturating_add(levels);
            if levels > 0 {
                p.notification = format!(
                    "⭐ Level Up! Stufe {} (+{} Stat-Punkte, +{} Fertigkeitspunkte)",
                    p.level,
                    levels * crate::progression::STAT_POINTS_PER_LEVEL,
                    levels,
                );
            } else if let Some(label) = killed_label {
                p.notification = format!("⚔️ {} getötet (+{} XP, +{} Gold)", label, total_xp, total_gold);
            }
        }
    }

    ActionResult { success: true, message: format!("{} gewirkt", def.name) }
}

/// Decrement skill cooldowns and active buff timers — call once per tick.
pub fn tick_player_skill_timers(state: &mut GameState) {
    for p in state.players.values_mut() {
        for cd in p.skill_cooldowns.values_mut() {
            if *cd > 0 { *cd -= 1; }
        }
        p.skill_cooldowns.retain(|_, cd| *cd > 0);
        for t in p.active_buffs.values_mut() {
            if *t > 0 { *t -= 1; }
        }
        p.active_buffs.retain(|_, t| *t > 0);
    }
    let _ = HashMap::<String, u32>::new(); // silence unused-import warning if any
}

struct ZoneCoords {
    x: f64,
    z: f64,
    zone: crate::world::ZoneId,
}
