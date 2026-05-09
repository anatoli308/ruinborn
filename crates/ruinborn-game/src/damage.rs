//! Damage model — Phase 3.
//!
//! 6 damage types × 5 tags. Every hit (player attack, skill cast, enemy hit,
//! DoT tick) is expressed as a [`DamageInstance`]. Targets carry [`Resistances`]
//! (one percentage per type, capped at 75%) and a list of active [`DotInstance`]s
//! that tick down each game tick.
//!
//! ```text
//! Types: Physical | Fire | Cold | Lightning | Poison | Magical
//! Tags : Melee    | Ranged | Spell | Summoning | Trap
//! ```

use serde::{Deserialize, Serialize};

/// The six damage types. Each maps 1:1 to a resistance channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    Physical,
    Fire,
    Cold,
    Lightning,
    Poison,
    Magical,
}

/// Orthogonal source classification. A single hit may carry multiple tags
/// (e.g. a thrown poison dagger is `Ranged + Poison(type) + Trap-like`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageTag {
    Melee,
    Ranged,
    Spell,
    Summoning,
    Trap,
}

/// A single damage event. Carries the payload + classification — no target ref.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageInstance {
    pub amount: f64,
    pub damage_type: DamageType,
    pub tags: Vec<DamageTag>,
}

impl DamageInstance {
    pub fn new(damage_type: DamageType, amount: f64, tags: &[DamageTag]) -> Self {
        Self { amount, damage_type, tags: tags.to_vec() }
    }
    pub fn physical_melee(amount: f64) -> Self {
        Self::new(DamageType::Physical, amount, &[DamageTag::Melee])
    }
    pub fn has_tag(&self, tag: DamageTag) -> bool {
        self.tags.iter().any(|t| *t == tag)
    }
}

/// Per-type damage reduction in percent (0..=75 in practice).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Resistances {
    pub physical: f64,
    pub fire: f64,
    pub cold: f64,
    pub lightning: f64,
    pub poison: f64,
    pub magical: f64,
}

/// Hard cap to keep glass cannons from going invulnerable.
pub const MAX_RESIST: f64 = 75.0;

impl Resistances {
    pub fn get(&self, t: DamageType) -> f64 {
        let raw = match t {
            DamageType::Physical => self.physical,
            DamageType::Fire => self.fire,
            DamageType::Cold => self.cold,
            DamageType::Lightning => self.lightning,
            DamageType::Poison => self.poison,
            DamageType::Magical => self.magical,
        };
        raw.clamp(-100.0, MAX_RESIST)
    }
    /// Apply resistance to a raw amount.
    pub fn apply(&self, dmg: &DamageInstance) -> f64 {
        let r = self.get(dmg.damage_type);
        let mult = (1.0 - r / 100.0).max(0.0);
        (dmg.amount * mult).max(0.0)
    }
}

/// A damage-over-time effect. Applied per tick until `ticks_remaining` hits 0.
/// `tags` are inherited from the originating hit so a poison from a trap still
/// keeps the `Trap` flavor for analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotInstance {
    pub damage_type: DamageType,
    pub damage_per_tick: f64,
    pub ticks_remaining: u32,
    pub tags: Vec<DamageTag>,
}

impl DotInstance {
    pub fn poison(dps: f64, ticks: u32) -> Self {
        Self {
            damage_type: DamageType::Poison,
            damage_per_tick: dps,
            ticks_remaining: ticks,
            tags: vec![DamageTag::Spell],
        }
    }
}

/// Tick all DoTs once: returns total damage dealt (post-resist), drops expired entries.
pub fn tick_dots(dots: &mut Vec<DotInstance>, resistances: &Resistances) -> f64 {
    let mut total = 0.0;
    for d in dots.iter_mut() {
        let inst = DamageInstance::new(d.damage_type, d.damage_per_tick, &d.tags);
        total += resistances.apply(&inst);
        if d.ticks_remaining > 0 {
            d.ticks_remaining -= 1;
        }
    }
    dots.retain(|d| d.ticks_remaining > 0);
    total
}
