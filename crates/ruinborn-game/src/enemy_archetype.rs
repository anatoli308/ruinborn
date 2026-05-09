//! JSON-driven enemy archetype registry.
//!
//! All enemy stats (HP, damage, move speed, resistances, attack type, pack
//! size, …) live in [`data/enemies.json`]. The bundled file is embedded at
//! compile time via `include_str!` so the game can boot without external
//! files; the server may also call [`load_archetypes_from_str`] at startup
//! with a user-edited copy to override the defaults.
//!
//! Adding a new monster:
//! 1. Append an archetype block to `enemies.json`.
//! 2. Reference its `id` in `spawn_rules` for any zone-kind it should appear in.
//!
//! No Rust changes required for value tweaks — restart the server to reload.

use std::collections::HashMap;
use std::sync::OnceLock;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::damage::{DamageType, Resistances};
use crate::world::ZoneKind;

/// Bundled defaults — recompiled into the binary so we always have a fallback.
const BUNDLED_JSON: &str = include_str!("../data/enemies.json");

/// Optional poison rider applied by a melee hit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PoisonOnHit {
    /// Damage applied per combat tick (20 Hz).
    pub damage_per_tick: f64,
    /// Number of combat ticks the DoT lasts.
    pub ticks: u32,
}

/// Which AI engine drives this archetype's per-tick behaviour.
///
/// - `SimpleChase`: hard-coded Idle → Chase → Attack state machine in
///   `combat::tick_enemies` (default; backwards compatible).
/// - `Goap`: data-driven planner using `crate::ai::goap`. Requires a
///   matching agent definition in `data/goap/agents.json` (looked up by
///   the archetype's `id`). Falls back to `SimpleChase` at spawn time
///   if no agent config is registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AiKind {
    #[default]
    SimpleChase,
    Goap,
}

/// Per-monster stat block. Loaded from JSON; immutable at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyArchetype {
    /// Stable identifier referenced by `Enemy.kind` and `spawn_rules`.
    pub id: String,
    /// Display name shown in combat log / kill notifications.
    pub label: String,
    /// Client atlas key (rendered as `assets/npc/atlases/{atlas}.json`).
    pub atlas: String,

    pub max_hp_base: f64,
    pub max_hp_per_level: f64,
    pub damage_base: f64,
    pub damage_per_level: f64,
    /// World units moved per combat tick (20 Hz). 0.075 = 1.5 u/s.
    pub move_speed: f64,

    pub xp_base: u64,
    pub xp_per_level: u64,
    pub gold_min_base: u32,
    pub gold_min_per_level: u32,
    pub gold_max_base: u32,
    pub gold_max_per_level: u32,

    pub attack_damage_type: DamageType,
    /// Combat ticks between melee swings.
    pub attack_cooldown_ticks: u32,
    pub poison_on_hit: Option<PoisonOnHit>,

    pub resistances: Resistances,

    /// Pack-spawn metadata — when a fresh pack rolls this archetype, this many
    /// members spawn clustered around a single anchor point.
    pub pack_size_min: u32,
    pub pack_size_max: u32,
    pub pack_radius: f64,

    /// Which AI engine drives this archetype. Optional in JSON;
    /// defaults to `simple_chase` for backwards compatibility.
    #[serde(default)]
    pub ai: AiKind,
}

impl EnemyArchetype {
    pub fn max_hp(&self, level: u32) -> f64 {
        self.max_hp_base + self.max_hp_per_level * level as f64
    }
    pub fn damage(&self, level: u32) -> f64 {
        self.damage_base + self.damage_per_level * level as f64
    }
    pub fn xp_reward(&self, level: u32) -> u64 {
        self.xp_base + self.xp_per_level * level as u64
    }
    pub fn gold_min(&self, level: u32) -> u32 {
        self.gold_min_base + self.gold_min_per_level * level
    }
    pub fn gold_max(&self, level: u32) -> u32 {
        self.gold_max_base + self.gold_max_per_level * level
    }
    pub fn pack_size(&self, rng: &mut impl Rng) -> u32 {
        if self.pack_size_max <= self.pack_size_min {
            self.pack_size_min.max(1)
        } else {
            rng.gen_range(self.pack_size_min..=self.pack_size_max).max(1)
        }
    }
}

/// One entry inside `spawn_rules.<zone_kind>` — a weighted archetype pick
/// with a per-zone level range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRule {
    pub id: String,
    pub weight: f64,
    pub level_min: u32,
    pub level_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpawnRulesFile {
    #[serde(default)]
    pub wilderness: Vec<SpawnRule>,
    #[serde(default)]
    pub dungeon: Vec<SpawnRule>,
    #[serde(default)]
    pub town: Vec<SpawnRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnemiesFile {
    pub archetypes: Vec<EnemyArchetype>,
    pub spawn_rules: SpawnRulesFile,
}

#[derive(Debug)]
pub struct ArchetypeRegistry {
    pub by_id: HashMap<String, EnemyArchetype>,
    pub spawn_wilderness: Vec<SpawnRule>,
    pub spawn_dungeon: Vec<SpawnRule>,
    pub spawn_town: Vec<SpawnRule>,
}

impl ArchetypeRegistry {
    pub fn rules_for(&self, kind: ZoneKind) -> &[SpawnRule] {
        match kind {
            ZoneKind::Wilderness => &self.spawn_wilderness,
            ZoneKind::Dungeon => &self.spawn_dungeon,
            ZoneKind::Town => &self.spawn_town,
        }
    }
}

static REGISTRY: OnceLock<ArchetypeRegistry> = OnceLock::new();

/// Parse a JSON document and overwrite the registry. Must be called at most
/// once — subsequent calls are ignored. Returns an error string if the JSON
/// references duplicate ids or spawn rules that don't resolve.
pub fn load_archetypes_from_str(json: &str) -> Result<(), String> {
    let parsed: EnemiesFile =
        serde_json::from_str(json).map_err(|e| format!("enemies.json parse error: {e}"))?;
    let mut by_id = HashMap::new();
    for arch in parsed.archetypes {
        if by_id.insert(arch.id.clone(), arch.clone()).is_some() {
            return Err(format!("duplicate enemy archetype id: {}", arch.id));
        }
    }
    let validate = |rules: &[SpawnRule], zone: &str| -> Result<(), String> {
        for r in rules {
            if !by_id.contains_key(&r.id) {
                return Err(format!(
                    "spawn_rules.{zone} references unknown archetype id `{}`",
                    r.id
                ));
            }
        }
        Ok(())
    };
    validate(&parsed.spawn_rules.wilderness, "wilderness")?;
    validate(&parsed.spawn_rules.dungeon, "dungeon")?;
    validate(&parsed.spawn_rules.town, "town")?;

    let registry = ArchetypeRegistry {
        by_id,
        spawn_wilderness: parsed.spawn_rules.wilderness,
        spawn_dungeon: parsed.spawn_rules.dungeon,
        spawn_town: parsed.spawn_rules.town,
    };
    let _ = REGISTRY.set(registry);
    Ok(())
}

fn registry() -> &'static ArchetypeRegistry {
    REGISTRY.get_or_init(|| {
        // First access without explicit init — load the bundled defaults.
        let parsed: EnemiesFile = serde_json::from_str(BUNDLED_JSON)
            .expect("bundled enemies.json must be valid (compile-time constant)");
        let mut by_id = HashMap::new();
        for arch in parsed.archetypes {
            by_id.insert(arch.id.clone(), arch);
        }
        ArchetypeRegistry {
            by_id,
            spawn_wilderness: parsed.spawn_rules.wilderness,
            spawn_dungeon: parsed.spawn_rules.dungeon,
            spawn_town: parsed.spawn_rules.town,
        }
    })
}

/// Look up an archetype by id. Panics if missing — archetypes referenced by
/// `Enemy.kind` are validated at spawn time via `pick_archetype_for_zone`.
pub fn archetype(id: &str) -> &'static EnemyArchetype {
    registry()
        .by_id
        .get(id)
        .unwrap_or_else(|| panic!("missing enemy archetype `{id}` (load_archetypes_from_str?)"))
}

/// `Some(arch)` iff registered. Use this on hot paths that may receive a
/// stale id from persisted data.
pub fn try_archetype(id: &str) -> Option<&'static EnemyArchetype> {
    registry().by_id.get(id)
}

/// Roll a (archetype, level) pair for a fresh spawn in the given zone-kind.
/// Returns `None` if no spawn rules exist (e.g. town).
pub fn pick_archetype_for_zone(
    zone_kind: ZoneKind,
    rng: &mut impl Rng,
) -> Option<(&'static EnemyArchetype, u32)> {
    let reg = registry();
    let rules = reg.rules_for(zone_kind);
    if rules.is_empty() {
        return None;
    }
    let total: f64 = rules.iter().map(|r| r.weight.max(0.0)).sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll = rng.gen_range(0.0..total);
    for rule in rules {
        let w = rule.weight.max(0.0);
        if roll < w {
            let arch = reg.by_id.get(&rule.id)?;
            let level = if rule.level_max <= rule.level_min {
                rule.level_min.max(1)
            } else {
                rng.gen_range(rule.level_min..=rule.level_max).max(1)
            };
            return Some((arch, level));
        }
        roll -= w;
    }
    None
}

/// Iterate every loaded archetype id (debug / admin tooling).
pub fn all_ids() -> Vec<&'static str> {
    registry()
        .by_id
        .keys()
        .map(|s| s.as_str())
        .collect()
}
