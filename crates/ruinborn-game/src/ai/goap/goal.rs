//! Goals — what an agent wants to be true about the world.

use serde::{Deserialize, Serialize};

use super::condition::Condition;
use super::world_state::WorldState;

/// A weighted desired state. Higher [`weight`] biases the planner toward
/// pursuing this goal when multiple are unsatisfied.
///
/// A goal is "achieved" when *all* of its conditions are satisfied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub label: String,
    /// Higher = more important. Used by the agent to pick which goal to
    /// solve when multiple are unmet.
    pub weight: f64,
    pub conditions: Vec<Condition>,
}

impl Goal {
    pub fn is_achieved(&self, world: &WorldState) -> bool {
        self.conditions.iter().all(|c| c.is_satisfied(world))
    }
}

#[cfg(test)]
mod tests {
    use super::super::condition::Comparison;
    use super::super::world_state::SenseValue;
    use super::*;

    #[test]
    fn goal_achieved_when_all_conditions_hold() {
        let goal = Goal {
            id: "kill".into(),
            label: "Kill".into(),
            weight: 1.0,
            conditions: vec![Condition {
                key: "target_dead".into(),
                op: Comparison::Equal,
                value: SenseValue::Bool(true),
            }],
        };
        let mut w = WorldState::new();
        w.insert("target_dead".into(), SenseValue::Bool(true));
        assert!(goal.is_achieved(&w));
    }

    #[test]
    fn goal_not_achieved_when_one_missing() {
        let goal = Goal {
            id: "kill".into(),
            label: "Kill".into(),
            weight: 1.0,
            conditions: vec![Condition {
                key: "target_dead".into(),
                op: Comparison::Equal,
                value: SenseValue::Bool(true),
            }],
        };
        assert!(!goal.is_achieved(&WorldState::new()));
    }
}
