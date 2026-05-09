//! World keys and sense values.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Stable, dotless identifier for a world fact (e.g. `"has_target"`,
/// `"in_attack_range"`, `"hp_percent"`).
///
/// We keep this as a `String` (rather than an enum) so new keys can be
/// introduced from JSON without a Rust recompile — same authoring story
/// as `enemy_archetype.rs`.
pub type WorldKey = String;

/// Current value of a [`WorldKey`]. CrashKonijn supports a wider type set
/// (vector, object, …); we start with the two that 95 % of conditions
/// actually need. More variants are easy to add later.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SenseValue {
    Bool(bool),
    Int(i32),
}

impl SenseValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SenseValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            SenseValue::Int(i) => Some(*i),
            _ => None,
        }
    }
}

/// Full per-agent world state — the input the planner reasons over and
/// effects mutate.
pub type WorldState = HashMap<WorldKey, SenseValue>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sense_value_round_trips_bool() {
        let v = SenseValue::Bool(true);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "true");
        let parsed: SenseValue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.as_bool(), Some(true));
    }

    #[test]
    fn sense_value_round_trips_int() {
        let v = SenseValue::Int(42);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "42");
        let parsed: SenseValue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.as_int(), Some(42));
    }
}
