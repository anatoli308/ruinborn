//! GOAP planner — A* search through action chains.
//!
//! ## Algorithm
//!
//! Forward search in *world-state space*. Each node is a snapshot of the
//! simulated world; an edge is one [`GoapAction`] whose preconditions
//! hold in that snapshot, applied via its effects. The search succeeds
//! when a node satisfies the [`Goal`].
//!
//! Forward search (instead of CrashKonijn's backward variant) keeps
//! preconditions concrete at every step — we never have to invent
//! "missing" facts during planning, so the resulting plan is guaranteed
//! executable from the agent's current world.
//!
//! ## Heuristic
//!
//! `h(state) = number of goal conditions still unsatisfied`. Admissible
//! when every action satisfies at most one condition (true for our
//! authored data); slightly inadmissible otherwise but in practice
//! produces optimal plans on the small graphs we generate.
//!
//! ## Cost model
//!
//! `g(state) = sum of action.cost along the path`. The runtime may add
//! dynamic cost later (distance to target, etc.) without changing the
//! resolver.
//!
//! ## Bounds
//!
//! Hard cap of [`MAX_NODES`] expanded states. Authored agents currently
//! stay well under 100. The cap exists so a buggy effect chain can't
//! freeze the tick loop.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use serde::{Deserialize, Serialize};

use super::action::GoapAction;
use super::condition::Condition;
use super::config::GoapAgentConfig;
use super::goal::Goal;
use super::world_state::{SenseValue, WorldState};

/// Safety bound on A* expansion. ~10× larger than anything our authored
/// graphs generate today; raises a structural alarm rather than hanging.
pub const MAX_NODES: usize = 1024;

/// Reason a planning attempt did not produce a usable plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanFailure {
    /// Goal was already satisfied — caller should pick another goal.
    AlreadyAchieved,
    /// No action sequence reachable from the start state satisfies the goal.
    Unreachable,
    /// Search hit [`MAX_NODES`] before finding a plan. Authoring bug.
    SearchExhausted,
}

/// Successful plan: an ordered list of action ids the agent should run
/// from current state to satisfy the goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub goal_id: String,
    pub actions: Vec<String>,
    /// Cumulative `g`-cost of the chosen path.
    pub total_cost: u64,
}

impl Plan {
    pub fn first_action(&self) -> Option<&str> {
        self.actions.first().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

/// Plan a path from `start` toward `goal` using `agent`'s actions.
///
/// Returns `Ok(Plan)` with at least one action, or a [`PlanFailure`].
pub fn plan(
    agent: &GoapAgentConfig,
    goal: &Goal,
    start: &WorldState,
) -> Result<Plan, PlanFailure> {
    if goal.is_achieved(start) {
        return Err(PlanFailure::AlreadyAchieved);
    }

    // Open set: min-heap on f = g + h.
    let mut open: BinaryHeap<Node> = BinaryHeap::new();
    // Visited states keyed by canonical world hash → best `g` so far.
    let mut best_g: HashMap<u64, u64> = HashMap::new();
    // Path reconstruction: child-hash → (parent-hash, action-id, g-at-child).
    let mut came_from: HashMap<u64, (u64, String)> = HashMap::new();

    let start_hash = hash_world(start);
    let start_h = heuristic(&goal.conditions, start);
    best_g.insert(start_hash, 0);
    open.push(Node {
        f: start_h as u64,
        g: 0,
        hash: start_hash,
        state: start.clone(),
    });

    let mut expanded = 0usize;

    while let Some(current) = open.pop() {
        // Goal test on dequeue (consistent with A* on integer costs).
        if conditions_satisfied(&goal.conditions, &current.state) {
            return Ok(reconstruct(
                goal.id.clone(),
                current.hash,
                start_hash,
                &came_from,
                current.g,
            ));
        }

        expanded += 1;
        if expanded > MAX_NODES {
            return Err(PlanFailure::SearchExhausted);
        }

        // Stale heap entry — a cheaper path to this state was already processed.
        if let Some(&seen) = best_g.get(&current.hash) {
            if current.g > seen {
                continue;
            }
        }

        for action in &agent.actions {
            if !preconditions_hold(action, &current.state) {
                continue;
            }
            // Reject no-op actions: must change at least one fact.
            let mut next = current.state.clone();
            for eff in &action.effects {
                eff.apply(&mut next);
            }
            if next == current.state {
                continue;
            }

            let next_hash = hash_world(&next);
            // Action cost rounded to integer ticks-equivalent. We keep
            // costs as `u64` internally so heap comparisons are total.
            let step_cost = action.cost.max(0.0).round() as u64;
            // Floor of 1 prevents zero-cost loops.
            let step_cost = step_cost.max(1);
            let tentative_g = current.g.saturating_add(step_cost);

            if let Some(&prior) = best_g.get(&next_hash) {
                if tentative_g >= prior {
                    continue;
                }
            }

            best_g.insert(next_hash, tentative_g);
            came_from.insert(next_hash, (current.hash, action.id.clone()));

            let h = heuristic(&goal.conditions, &next) as u64;
            open.push(Node {
                f: tentative_g.saturating_add(h),
                g: tentative_g,
                hash: next_hash,
                state: next,
            });
        }
    }

    Err(PlanFailure::Unreachable)
}

fn preconditions_hold(action: &GoapAction, world: &WorldState) -> bool {
    action.preconditions.iter().all(|c| c.is_satisfied(world))
}

fn conditions_satisfied(conds: &[Condition], world: &WorldState) -> bool {
    conds.iter().all(|c| c.is_satisfied(world))
}

fn heuristic(conds: &[Condition], world: &WorldState) -> usize {
    conds.iter().filter(|c| !c.is_satisfied(world)).count()
}

fn reconstruct(
    goal_id: String,
    end: u64,
    start: u64,
    came_from: &HashMap<u64, (u64, String)>,
    total_cost: u64,
) -> Plan {
    let mut actions = Vec::new();
    let mut cursor = end;
    while cursor != start {
        let Some((parent, action_id)) = came_from.get(&cursor) else {
            // Should be unreachable: any non-start node has a parent recorded.
            break;
        };
        actions.push(action_id.clone());
        cursor = *parent;
    }
    actions.reverse();
    Plan {
        goal_id,
        actions,
        total_cost,
    }
}

// ---------- Internal node + canonical hash ----------

#[derive(Debug)]
struct Node {
    f: u64,
    g: u64,
    hash: u64,
    state: WorldState,
}

// Min-heap via reversed Ord on `f`.
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.g.cmp(&self.g))
            .then_with(|| self.hash.cmp(&other.hash))
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.g == other.g && self.hash == other.hash
    }
}
impl Eq for Node {}

/// Canonical hash of a [`WorldState`] independent of HashMap iteration
/// order. Uses a sorted (key, value) sequence + FxHash-style mixing.
fn hash_world(state: &WorldState) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut entries: Vec<(&str, u64)> = state
        .iter()
        .map(|(k, v)| (k.as_str(), encode_value(*v)))
        .collect();
    entries.sort_unstable_by(|a, b| a.0.cmp(b.0));

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (k, v) in entries {
        k.hash(&mut hasher);
        v.hash(&mut hasher);
    }
    hasher.finish()
}

fn encode_value(v: SenseValue) -> u64 {
    match v {
        SenseValue::Bool(false) => 0,
        SenseValue::Bool(true) => 1,
        // Tag ints separately from bools to avoid collision (Int(0) vs Bool(false)).
        SenseValue::Int(i) => 0x1_0000_0000u64 | (i as u32 as u64),
    }
}

#[cfg(test)]
mod tests {
    use super::super::action::GoapAction;
    use super::super::condition::{Comparison, Condition, Effect, EffectOp};
    use super::super::config::GoapAgentConfig;
    use super::super::goal::Goal;
    use super::super::world_state::{SenseValue, WorldState};
    use super::*;

    fn cond_eq(key: &str, val: SenseValue) -> Condition {
        Condition {
            key: key.into(),
            op: Comparison::Equal,
            value: val,
        }
    }

    fn eff_set(key: &str, val: SenseValue) -> Effect {
        Effect {
            key: key.into(),
            op: EffectOp::Set,
            value: val,
        }
    }

    fn act(id: &str, cost: f64, pre: Vec<Condition>, eff: Vec<Effect>) -> GoapAction {
        GoapAction {
            id: id.into(),
            label: id.into(),
            cost,
            preconditions: pre,
            effects: eff,
            target_key: None,
            in_range: 0.0,
            ticks_to_perform: 1,
            behaviour: "noop".into(),
        }
    }

    fn agent(actions: Vec<GoapAction>) -> GoapAgentConfig {
        GoapAgentConfig {
            id: "test".into(),
            initial_world: WorldState::new(),
            goals: vec![],
            actions,
        }
    }

    #[test]
    fn already_achieved_returns_error() {
        let g = Goal {
            id: "g".into(),
            label: "g".into(),
            weight: 1.0,
            conditions: vec![cond_eq("done", SenseValue::Bool(true))],
        };
        let mut w = WorldState::new();
        w.insert("done".into(), SenseValue::Bool(true));

        let result = plan(&agent(vec![]), &g, &w);
        assert_eq!(result, Err(PlanFailure::AlreadyAchieved));
    }

    #[test]
    fn unreachable_when_no_action_satisfies_goal() {
        let g = Goal {
            id: "g".into(),
            label: "g".into(),
            weight: 1.0,
            conditions: vec![cond_eq("done", SenseValue::Bool(true))],
        };
        let actions = vec![act(
            "useless",
            1.0,
            vec![],
            vec![eff_set("other", SenseValue::Bool(true))],
        )];

        let result = plan(&agent(actions), &g, &WorldState::new());
        assert_eq!(result, Err(PlanFailure::Unreachable));
    }

    #[test]
    fn finds_single_step_plan() {
        let g = Goal {
            id: "g".into(),
            label: "g".into(),
            weight: 1.0,
            conditions: vec![cond_eq("done", SenseValue::Bool(true))],
        };
        let actions = vec![act(
            "do_it",
            1.0,
            vec![],
            vec![eff_set("done", SenseValue::Bool(true))],
        )];

        let plan_ = plan(&agent(actions), &g, &WorldState::new()).expect("plan");
        assert_eq!(plan_.actions, vec!["do_it"]);
        assert_eq!(plan_.total_cost, 1);
    }

    #[test]
    fn finds_multi_step_plan_in_correct_order() {
        // Goal: target_dead = true.
        // Need: has_target -> in_range -> attack.
        let g = Goal {
            id: "kill".into(),
            label: "Kill".into(),
            weight: 1.0,
            conditions: vec![cond_eq("target_dead", SenseValue::Bool(true))],
        };
        let actions = vec![
            act(
                "acquire",
                1.0,
                vec![cond_eq("has_target", SenseValue::Bool(false))],
                vec![eff_set("has_target", SenseValue::Bool(true))],
            ),
            act(
                "approach",
                1.0,
                vec![cond_eq("has_target", SenseValue::Bool(true))],
                vec![eff_set("in_range", SenseValue::Bool(true))],
            ),
            act(
                "attack",
                1.0,
                vec![
                    cond_eq("has_target", SenseValue::Bool(true)),
                    cond_eq("in_range", SenseValue::Bool(true)),
                ],
                vec![eff_set("target_dead", SenseValue::Bool(true))],
            ),
        ];

        let mut w = WorldState::new();
        w.insert("has_target".into(), SenseValue::Bool(false));
        w.insert("in_range".into(), SenseValue::Bool(false));
        w.insert("target_dead".into(), SenseValue::Bool(false));

        let plan_ = plan(&agent(actions), &g, &w).expect("plan");
        assert_eq!(plan_.actions, vec!["acquire", "approach", "attack"]);
        assert_eq!(plan_.total_cost, 3);
    }

    #[test]
    fn picks_cheaper_branch_when_two_paths_exist() {
        // Two ways to set `done`: cheap (cost 1) vs expensive (cost 10).
        let g = Goal {
            id: "g".into(),
            label: "g".into(),
            weight: 1.0,
            conditions: vec![cond_eq("done", SenseValue::Bool(true))],
        };
        let actions = vec![
            act(
                "expensive",
                10.0,
                vec![],
                vec![eff_set("done", SenseValue::Bool(true))],
            ),
            act(
                "cheap",
                1.0,
                vec![],
                vec![eff_set("done", SenseValue::Bool(true))],
            ),
        ];

        let plan_ = plan(&agent(actions), &g, &WorldState::new()).expect("plan");
        assert_eq!(plan_.actions, vec!["cheap"]);
        assert_eq!(plan_.total_cost, 1);
    }

    #[test]
    fn ignores_actions_whose_preconditions_fail() {
        let g = Goal {
            id: "g".into(),
            label: "g".into(),
            weight: 1.0,
            conditions: vec![cond_eq("done", SenseValue::Bool(true))],
        };
        let actions = vec![
            // Locked behind a precondition no other action satisfies.
            act(
                "locked",
                1.0,
                vec![cond_eq("never", SenseValue::Bool(true))],
                vec![eff_set("done", SenseValue::Bool(true))],
            ),
            act(
                "open",
                5.0,
                vec![],
                vec![eff_set("done", SenseValue::Bool(true))],
            ),
        ];

        let plan_ = plan(&agent(actions), &g, &WorldState::new()).expect("plan");
        assert_eq!(plan_.actions, vec!["open"]);
    }

    #[test]
    fn no_op_actions_are_skipped_to_prevent_infinite_loop() {
        // Action with effects that don't change the world should not be
        // considered. Without the no-op guard, A* could loop forever.
        let g = Goal {
            id: "g".into(),
            label: "g".into(),
            weight: 1.0,
            conditions: vec![cond_eq("done", SenseValue::Bool(true))],
        };
        let actions = vec![
            act(
                "noop",
                1.0,
                vec![],
                vec![eff_set("flag", SenseValue::Bool(false))],
            ),
            act(
                "win",
                2.0,
                vec![],
                vec![eff_set("done", SenseValue::Bool(true))],
            ),
        ];

        let mut w = WorldState::new();
        w.insert("flag".into(), SenseValue::Bool(false));

        let plan_ = plan(&agent(actions), &g, &w).expect("plan");
        assert_eq!(plan_.actions, vec!["win"]);
    }

    #[test]
    fn bundled_zombie_kill_plan_is_solvable() {
        let agent = super::super::config::try_agent_config("zombie").expect("zombie agent");
        let goal = agent.goal("kill_target").expect("kill_target goal");
        let plan_ = plan(agent, goal, &agent.initial_world).expect("plan");
        // Acquire → chase → attack.
        assert_eq!(
            plan_.actions,
            vec!["acquire_target", "chase_target", "melee_attack"]
        );
    }

    #[test]
    fn bundled_skeleton_kill_plan_is_solvable() {
        let agent = super::super::config::try_agent_config("skeleton").expect("skeleton agent");
        let goal = agent.goal("kill_target").expect("kill_target goal");
        let plan_ = plan(agent, goal, &agent.initial_world).expect("plan");
        assert_eq!(
            plan_.actions,
            vec!["acquire_target", "chase_target", "melee_attack"]
        );
    }
}
