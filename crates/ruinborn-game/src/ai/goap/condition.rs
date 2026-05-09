//! Conditions (read predicates) and Effects (write predicates) over
//! the world state.

use serde::{Deserialize, Serialize};

use super::world_state::{SenseValue, WorldKey, WorldState};

/// Comparison operator used by [`Condition`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
}

/// A read predicate over the world state. Used by goals (target state)
/// and actions (preconditions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub key: WorldKey,
    pub op: Comparison,
    pub value: SenseValue,
}

impl Condition {
    /// Evaluate this condition against a world snapshot. Missing keys
    /// always evaluate to `false` — same semantics as CrashKonijn:
    /// "no sensor wrote this yet, so we can't claim it holds".
    pub fn is_satisfied(&self, world: &WorldState) -> bool {
        let Some(actual) = world.get(&self.key) else {
            return false;
        };
        compare(actual, self.op, &self.value)
    }
}

fn compare(actual: &SenseValue, op: Comparison, expected: &SenseValue) -> bool {
    match (actual, expected) {
        (SenseValue::Bool(a), SenseValue::Bool(b)) => match op {
            Comparison::Equal => a == b,
            Comparison::NotEqual => a != b,
            // Bool ordering is not meaningful — return false rather than panic.
            _ => false,
        },
        (SenseValue::Int(a), SenseValue::Int(b)) => match op {
            Comparison::Equal => a == b,
            Comparison::NotEqual => a != b,
            Comparison::GreaterThan => a > b,
            Comparison::LessThan => a < b,
            Comparison::GreaterOrEqual => a >= b,
            Comparison::LessOrEqual => a <= b,
        },
        // Type mismatch — author error in JSON; fail closed.
        _ => false,
    }
}

/// What an [`Effect`] does to a world value when an action completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectOp {
    /// Overwrite the key with `value`.
    Set,
    /// Add `value` (Int only). Booleans behave as `Set`.
    Increase,
    /// Subtract `value` (Int only). Booleans behave as `Set`.
    Decrease,
}

/// A write predicate — declares what an action will change about the
/// world if it succeeds. Used by the planner during back-chaining and by
/// the runtime to update the simulated world after an action runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub key: WorldKey,
    pub op: EffectOp,
    pub value: SenseValue,
}

impl Effect {
    /// Apply this effect in-place to the given world state.
    pub fn apply(&self, world: &mut WorldState) {
        match (self.op, self.value) {
            (EffectOp::Set, v) => {
                world.insert(self.key.clone(), v);
            }
            (EffectOp::Increase, SenseValue::Int(delta)) => {
                let current = world
                    .get(&self.key)
                    .and_then(SenseValue::as_int)
                    .unwrap_or(0);
                world.insert(self.key.clone(), SenseValue::Int(current + delta));
            }
            (EffectOp::Decrease, SenseValue::Int(delta)) => {
                let current = world
                    .get(&self.key)
                    .and_then(SenseValue::as_int)
                    .unwrap_or(0);
                world.insert(self.key.clone(), SenseValue::Int(current - delta));
            }
            // Increase/Decrease on a bool is meaningless — treat as Set.
            (EffectOp::Increase | EffectOp::Decrease, v @ SenseValue::Bool(_)) => {
                world.insert(self.key.clone(), v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> WorldState {
        let mut w = WorldState::new();
        w.insert("has_target".into(), SenseValue::Bool(true));
        w.insert("hp_percent".into(), SenseValue::Int(50));
        w
    }

    #[test]
    fn condition_bool_equal() {
        let c = Condition {
            key: "has_target".into(),
            op: Comparison::Equal,
            value: SenseValue::Bool(true),
        };
        assert!(c.is_satisfied(&world()));
    }

    #[test]
    fn condition_int_greater_than() {
        let c = Condition {
            key: "hp_percent".into(),
            op: Comparison::GreaterThan,
            value: SenseValue::Int(25),
        };
        assert!(c.is_satisfied(&world()));
    }

    #[test]
    fn condition_missing_key_is_false() {
        let c = Condition {
            key: "nope".into(),
            op: Comparison::Equal,
            value: SenseValue::Bool(true),
        };
        assert!(!c.is_satisfied(&world()));
    }

    #[test]
    fn effect_increase_int() {
        let mut w = world();
        Effect {
            key: "hp_percent".into(),
            op: EffectOp::Increase,
            value: SenseValue::Int(10),
        }
        .apply(&mut w);
        assert_eq!(w.get("hp_percent").unwrap().as_int(), Some(60));
    }

    #[test]
    fn effect_set_bool() {
        let mut w = world();
        Effect {
            key: "has_target".into(),
            op: EffectOp::Set,
            value: SenseValue::Bool(false),
        }
        .apply(&mut w);
        assert_eq!(w.get("has_target").unwrap().as_bool(), Some(false));
    }
}
