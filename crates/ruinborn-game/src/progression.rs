//! D2-style player progression: level, XP, HP, mana, stat points.
//!
//! Stats follow Diablo 2:
//! - Strength    -> melee damage bonus
//! - Dexterity   -> ranged damage / hit rating
//! - Vitality    -> +life per point
//! - Energy      -> +mana per point
//!
//! Each level grants `STAT_POINTS_PER_LEVEL` unspent points the player can allocate.

use serde::{Deserialize, Serialize};

pub const STARTING_LEVEL: u32 = 1;
pub const MAX_LEVEL: u32 = 99;
pub const STAT_POINTS_PER_LEVEL: u32 = 5;
pub const BASE_HP: f64 = 50.0;
pub const HP_PER_VITALITY: f64 = 2.0;
pub const HP_PER_LEVEL: f64 = 10.0;
pub const BASE_MANA: f64 = 15.0;
pub const MANA_PER_ENERGY: f64 = 1.5;
pub const MANA_PER_LEVEL: f64 = 1.0;

/// XP needed to reach `level` from `level-1`. D2-ish curve, not 1:1.
pub fn xp_for_next_level(level: u32) -> u64 {
    // Polynomial, gentle early game, tougher later.
    let l = level.max(1) as f64;
    (50.0 * l.powf(1.7)).round() as u64
}

/// Per-player Diablo-style attribute block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub strength: u32,
    pub dexterity: u32,
    pub vitality: u32,
    pub energy: u32,
}

impl Default for Stats {
    fn default() -> Self {
        // Generic D2-ish starter spread.
        Self { strength: 10, dexterity: 10, vitality: 10, energy: 10 }
    }
}

impl Stats {
    pub fn max_hp(&self, level: u32) -> f64 {
        BASE_HP + HP_PER_LEVEL * (level.saturating_sub(1) as f64) + HP_PER_VITALITY * self.vitality as f64
    }
    pub fn max_mana(&self, level: u32) -> f64 {
        BASE_MANA + MANA_PER_LEVEL * (level.saturating_sub(1) as f64) + MANA_PER_ENERGY * self.energy as f64
    }
    /// Base melee damage contribution from STR.
    pub fn melee_bonus(&self) -> f64 {
        self.strength as f64 * 0.5
    }
}

/// Apply XP gain. Levels up potentially multiple times. Returns number of level-ups.
pub fn grant_xp(
    level: &mut u32,
    xp: &mut u64,
    xp_to_next: &mut u64,
    unspent_points: &mut u32,
    hp: &mut f64,
    max_hp: &mut f64,
    mana: &mut f64,
    max_mana: &mut f64,
    stats: &Stats,
    amount: u64,
) -> u32 {
    *xp = xp.saturating_add(amount);
    let mut levels_gained: u32 = 0;
    while *level < MAX_LEVEL && *xp >= *xp_to_next {
        *xp -= *xp_to_next;
        *level += 1;
        *unspent_points = unspent_points.saturating_add(STAT_POINTS_PER_LEVEL);
        *xp_to_next = xp_for_next_level(*level + 1);
        // Recompute max HP/mana, fully heal on level up (D2-style).
        *max_hp = stats.max_hp(*level);
        *max_mana = stats.max_mana(*level);
        *hp = *max_hp;
        *mana = *max_mana;
        levels_gained += 1;
    }
    levels_gained
}

/// Build the initial XP+HP+Mana block for a brand-new character.
pub fn starter_progression() -> StarterProgression {
    let stats = Stats::default();
    let level = STARTING_LEVEL;
    let max_hp = stats.max_hp(level);
    let max_mana = stats.max_mana(level);
    StarterProgression {
        level,
        xp: 0,
        xp_to_next: xp_for_next_level(level + 1),
        unspent_stat_points: 0,
        stats,
        hp: max_hp,
        max_hp,
        mana: max_mana,
        max_mana,
    }
}

/// Bundle returned by `starter_progression` to seed a new `PlayerState`.
pub struct StarterProgression {
    pub level: u32,
    pub xp: u64,
    pub xp_to_next: u64,
    pub unspent_stat_points: u32,
    pub stats: Stats,
    pub hp: f64,
    pub max_hp: f64,
    pub mana: f64,
    pub max_mana: f64,
}
