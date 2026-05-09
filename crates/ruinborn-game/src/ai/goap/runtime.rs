//! GOAP agent runtime — per-agent state machine that drives plan
//! selection, action progress, and replanning across ticks.
//!
//! ## Model
//!
//! Each agent owns one [`GoapAgentRuntime`]. The runtime is **pure
//! data + state transitions** — it does *not* touch the world, move
//! the agent, or deal damage. The caller (Phase 4 integration in
//! `combat.rs`) is responsible for:
//!
//! 1. Running sensors before each tick and writing the results into
//!    [`GoapAgentRuntime::world`].
//! 2. Calling [`GoapAgentRuntime::tick`] to get the [`RuntimeStep`].
//! 3. Executing the action's `behaviour` tag (move, attack, wander).
//! 4. Calling [`GoapAgentRuntime::complete_action`] when the runtime
//!    signalled `completing = true`.
//!
//! ## Per-tick cycle (Phase 4 will glue this together)
//!
//! ```text
//! sensors   →   runtime.world updated
//!     ↓
//! runtime.tick(agent)   →   RuntimeStep::Running { id, completing }
//!     ↓
//! caller dispatches `agent.action(id).behaviour`
//!     ↓
//! if completing → runtime.complete_action(agent)
//! ```
//!
//! ## Goal selection
//!
//! Highest-weight unsatisfied goal wins. Ties broken by goal-list
//! order (deterministic). When all goals are achieved, the runtime
//! returns [`RuntimeStep::Idle`] and the caller can park the agent.
//!
//! ## Replanning
//!
//! Triggered automatically when:
//! - No plan exists yet, or the plan is empty.
//! - The current goal is now achieved (early exit, pick a new goal).
//! - The next-up action's preconditions no longer hold.
//! - The caller invoked [`GoapAgentRuntime::force_replan`].
//!
//! Replanning is cheap on our authored graphs (~50 µs) but *not free*
//! — we only do it when one of the conditions above fires, not every
//! tick.

use serde::{Deserialize, Serialize};

use super::config::GoapAgentConfig;
use super::resolver::{plan, Plan, PlanFailure};
use super::world_state::WorldState;

/// Per-agent runtime state. Persisted across ticks; can be embedded on
/// `Enemy` (Phase 4) without further wrapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoapAgentRuntime {
    /// Current world snapshot — written by sensors before each tick.
    pub world: WorldState,
    /// Goal currently being pursued (`None` until first plan).
    pub current_goal: Option<String>,
    /// Cached plan; invalidated by replan triggers.
    pub current_plan: Option<Plan>,
    /// Index into `current_plan.actions` of the action currently running.
    pub action_step: usize,
    /// Combat ticks elapsed inside the current action.
    pub ticks_in_action: u32,
}

impl GoapAgentRuntime {
    /// Create a runtime seeded with the agent's authored
    /// `initial_world`. Sensors will overwrite these defaults on the
    /// first tick.
    pub fn new(agent: &GoapAgentConfig) -> Self {
        Self {
            world: agent.initial_world.clone(),
            current_goal: None,
            current_plan: None,
            action_step: 0,
            ticks_in_action: 0,
        }
    }

    /// Drive one combat tick. Returns the runtime's decision for this
    /// frame; the caller is responsible for executing the behaviour
    /// and calling [`Self::complete_action`] when signalled.
    pub fn tick(&mut self, agent: &GoapAgentConfig) -> RuntimeStep {
        if !self.plan_is_valid(agent) {
            self.replan(agent);
        }

        let Some(plan) = &self.current_plan else {
            return RuntimeStep::Idle;
        };

        let Some(action_id) = plan.actions.get(self.action_step) else {
            // Past the end — clear and bail; next tick will replan.
            self.clear();
            return RuntimeStep::Idle;
        };

        let Some(action) = agent.action(action_id) else {
            // Plan references a deleted action — recover via replan.
            self.clear();
            return RuntimeStep::Idle;
        };

        self.ticks_in_action = self.ticks_in_action.saturating_add(1);
        let completing = self.ticks_in_action >= action.ticks_to_perform.max(1);

        RuntimeStep::Running {
            action_id: action_id.clone(),
            ticks_in_action: self.ticks_in_action,
            completing,
        }
    }

    /// Apply the current action's effects to the runtime world and
    /// advance to the next step. Caller invokes this on the same tick
    /// `tick()` returned `completing = true`.
    pub fn complete_action(&mut self, agent: &GoapAgentConfig) {
        let Some(plan) = &self.current_plan else {
            return;
        };
        let Some(action_id) = plan.actions.get(self.action_step) else {
            return;
        };
        if let Some(action) = agent.action(action_id) {
            for eff in &action.effects {
                eff.apply(&mut self.world);
            }
        }
        self.action_step += 1;
        self.ticks_in_action = 0;

        // Plan complete — clear so next tick re-evaluates goals.
        if self.action_step
            >= self
                .current_plan
                .as_ref()
                .map(|p| p.actions.len())
                .unwrap_or(0)
        {
            self.clear();
        }
    }

    /// Force the runtime to discard its current plan. Use when an
    /// external event invalidates the plan (target died mid-chase,
    /// agent took burst damage and should flee, …).
    pub fn force_replan(&mut self) {
        self.clear();
    }

    /// Update one world fact (convenience for sensors that touch a
    /// single key per pass).
    pub fn set_world(&mut self, key: impl Into<String>, value: super::world_state::SenseValue) {
        self.world.insert(key.into(), value);
    }

    // ---------- internals ----------

    fn clear(&mut self) {
        self.current_goal = None;
        self.current_plan = None;
        self.action_step = 0;
        self.ticks_in_action = 0;
    }

    fn plan_is_valid(&self, agent: &GoapAgentConfig) -> bool {
        let Some(plan) = &self.current_plan else {
            return false;
        };
        let Some(goal_id) = &self.current_goal else {
            return false;
        };

        // Goal already achieved? Drop the plan and pick a new one.
        let Some(goal) = agent.goal(goal_id) else {
            return false;
        };
        if goal.is_achieved(&self.world) {
            return false;
        }

        // Step out of bounds → drop.
        if self.action_step >= plan.actions.len() {
            return false;
        }

        // Current action's preconditions must still hold against the
        // freshly-sensed world. Cheap check (typically 1–3 conds).
        let Some(action) = agent.action(&plan.actions[self.action_step]) else {
            return false;
        };
        action.preconditions.iter().all(|c| c.is_satisfied(&self.world))
    }

    fn replan(&mut self, agent: &GoapAgentConfig) {
        self.clear();

        // Sort goals by weight DESC, stable (preserves authoring order on ties).
        let mut goals: Vec<&super::goal::Goal> = agent.goals.iter().collect();
        goals.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));

        for goal in goals {
            if goal.is_achieved(&self.world) {
                continue;
            }
            match plan(agent, goal, &self.world) {
                Ok(p) if !p.is_empty() => {
                    self.current_goal = Some(goal.id.clone());
                    self.current_plan = Some(p);
                    return;
                }
                // Empty plan should be impossible (plan() returns AlreadyAchieved
                // for that), but guard defensively.
                Ok(_) => continue,
                Err(PlanFailure::AlreadyAchieved) => continue,
                // Unreachable / SearchExhausted → try next goal.
                Err(_) => continue,
            }
        }
    }
}

/// Decision the runtime made for the current tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeStep {
    /// No goal pursuable right now (all achieved, or no plan reachable).
    Idle,
    /// Caller should execute `action_id`'s behaviour. When `completing`
    /// is true, this is the final tick for the action — caller should
    /// invoke [`GoapAgentRuntime::complete_action`] after running its
    /// behaviour.
    Running {
        action_id: String,
        ticks_in_action: u32,
        completing: bool,
    },
}

impl RuntimeStep {
    pub fn action_id(&self) -> Option<&str> {
        match self {
            RuntimeStep::Running { action_id, .. } => Some(action_id.as_str()),
            RuntimeStep::Idle => None,
        }
    }

    pub fn is_completing(&self) -> bool {
        matches!(self, RuntimeStep::Running { completing: true, .. })
    }
}

#[cfg(test)]
mod tests {
    use super::super::action::GoapAction;
    use super::super::condition::{Comparison, Condition, Effect, EffectOp};
    use super::super::config::GoapAgentConfig;
    use super::super::goal::Goal;
    use super::super::world_state::SenseValue;
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

    fn act(
        id: &str,
        cost: f64,
        ticks: u32,
        pre: Vec<Condition>,
        eff: Vec<Effect>,
    ) -> GoapAction {
        GoapAction {
            id: id.into(),
            label: id.into(),
            cost,
            preconditions: pre,
            effects: eff,
            target_key: None,
            in_range: 0.0,
            ticks_to_perform: ticks,
            behaviour: id.into(),
        }
    }

    fn goal(id: &str, weight: f64, conds: Vec<Condition>) -> Goal {
        Goal {
            id: id.into(),
            label: id.into(),
            weight,
            conditions: conds,
        }
    }

    fn kill_agent() -> GoapAgentConfig {
        let mut initial = WorldState::new();
        initial.insert("has_target".into(), SenseValue::Bool(false));
        initial.insert("in_range".into(), SenseValue::Bool(false));
        initial.insert("target_dead".into(), SenseValue::Bool(false));

        GoapAgentConfig {
            id: "test".into(),
            initial_world: initial,
            goals: vec![goal(
                "kill",
                1.0,
                vec![cond_eq("target_dead", SenseValue::Bool(true))],
            )],
            actions: vec![
                act(
                    "acquire",
                    1.0,
                    1,
                    vec![cond_eq("has_target", SenseValue::Bool(false))],
                    vec![eff_set("has_target", SenseValue::Bool(true))],
                ),
                act(
                    "approach",
                    1.0,
                    1,
                    vec![cond_eq("has_target", SenseValue::Bool(true))],
                    vec![eff_set("in_range", SenseValue::Bool(true))],
                ),
                act(
                    "attack",
                    1.0,
                    3, // takes 3 ticks
                    vec![
                        cond_eq("has_target", SenseValue::Bool(true)),
                        cond_eq("in_range", SenseValue::Bool(true)),
                    ],
                    vec![eff_set("target_dead", SenseValue::Bool(true))],
                ),
            ],
        }
    }

    #[test]
    fn fresh_runtime_uses_initial_world() {
        let agent = kill_agent();
        let rt = GoapAgentRuntime::new(&agent);
        assert_eq!(
            rt.world.get("has_target"),
            Some(&SenseValue::Bool(false))
        );
    }

    #[test]
    fn first_tick_plans_and_runs_first_action() {
        let agent = kill_agent();
        let mut rt = GoapAgentRuntime::new(&agent);

        let step = rt.tick(&agent);
        match step {
            RuntimeStep::Running {
                action_id,
                ticks_in_action,
                completing,
            } => {
                assert_eq!(action_id, "acquire");
                assert_eq!(ticks_in_action, 1);
                assert!(completing, "1-tick action completes immediately");
            }
            other => panic!("expected Running, got {:?}", other),
        }
    }

    #[test]
    fn complete_action_applies_effects_and_advances() {
        let agent = kill_agent();
        let mut rt = GoapAgentRuntime::new(&agent);

        rt.tick(&agent);
        rt.complete_action(&agent);

        assert_eq!(rt.world.get("has_target"), Some(&SenseValue::Bool(true)));
        assert_eq!(rt.action_step, 1);
        assert_eq!(rt.ticks_in_action, 0);
    }

    #[test]
    fn full_kill_plan_runs_to_completion() {
        let agent = kill_agent();
        let mut rt = GoapAgentRuntime::new(&agent);

        // acquire (1 tick)
        assert!(rt.tick(&agent).is_completing());
        rt.complete_action(&agent);

        // approach (1 tick)
        assert!(rt.tick(&agent).is_completing());
        rt.complete_action(&agent);

        // attack: 3 ticks; only the third has completing=true
        let s1 = rt.tick(&agent);
        assert_eq!(s1.action_id(), Some("attack"));
        assert!(!s1.is_completing());
        let s2 = rt.tick(&agent);
        assert!(!s2.is_completing());
        let s3 = rt.tick(&agent);
        assert!(s3.is_completing());
        rt.complete_action(&agent);

        // Plan done → idle.
        assert_eq!(rt.tick(&agent), RuntimeStep::Idle);
        assert_eq!(rt.world.get("target_dead"), Some(&SenseValue::Bool(true)));
    }

    #[test]
    fn higher_weight_goal_is_preferred() {
        let mut agent = kill_agent();
        // Add a wander goal with lower weight.
        agent.goals.push(goal(
            "wander",
            0.1,
            vec![cond_eq("wandered", SenseValue::Bool(true))],
        ));
        agent.actions.push(act(
            "do_wander",
            1.0,
            1,
            vec![],
            vec![eff_set("wandered", SenseValue::Bool(true))],
        ));

        let mut rt = GoapAgentRuntime::new(&agent);
        let step = rt.tick(&agent);
        // Should pick the kill chain first, so first action is "acquire".
        assert_eq!(step.action_id(), Some("acquire"));
        assert_eq!(rt.current_goal.as_deref(), Some("kill"));
    }

    #[test]
    fn invalidated_precondition_triggers_replan() {
        let agent = kill_agent();
        let mut rt = GoapAgentRuntime::new(&agent);

        // Run acquire to completion → has_target = true.
        rt.tick(&agent);
        rt.complete_action(&agent);
        assert_eq!(rt.action_step, 1);

        // Sensor pass: target was lost externally → has_target=false again.
        rt.set_world("has_target", SenseValue::Bool(false));

        // Next tick should replan from scratch and start at "acquire" again.
        let step = rt.tick(&agent);
        assert_eq!(step.action_id(), Some("acquire"));
        assert_eq!(rt.action_step, 0);
    }

    #[test]
    fn goal_already_achieved_returns_idle() {
        let agent = kill_agent();
        let mut rt = GoapAgentRuntime::new(&agent);
        // Pretend a sensor saw target_dead = true before we even started.
        rt.set_world("target_dead", SenseValue::Bool(true));

        let step = rt.tick(&agent);
        assert_eq!(step, RuntimeStep::Idle);
    }

    #[test]
    fn force_replan_drops_current_plan() {
        let agent = kill_agent();
        let mut rt = GoapAgentRuntime::new(&agent);
        rt.tick(&agent);
        rt.complete_action(&agent);
        assert!(rt.current_plan.is_some());

        rt.force_replan();
        assert!(rt.current_plan.is_none());
        assert_eq!(rt.action_step, 0);
    }

    #[test]
    fn unreachable_goal_yields_idle_without_panicking() {
        // Goal with no satisfying action.
        let agent = GoapAgentConfig {
            id: "stuck".into(),
            initial_world: WorldState::new(),
            goals: vec![goal(
                "impossible",
                1.0,
                vec![cond_eq("magic", SenseValue::Bool(true))],
            )],
            actions: vec![act(
                "useless",
                1.0,
                1,
                vec![],
                vec![eff_set("other", SenseValue::Bool(true))],
            )],
        };
        let mut rt = GoapAgentRuntime::new(&agent);
        assert_eq!(rt.tick(&agent), RuntimeStep::Idle);
    }

    #[test]
    fn bundled_zombie_runtime_executes_full_kill_chain() {
        let agent = super::super::config::try_agent_config("zombie").expect("zombie");
        let mut rt = GoapAgentRuntime::new(agent);

        // Tick 1: acquire (1 tick)
        let s = rt.tick(agent);
        assert_eq!(s.action_id(), Some("acquire_target"));
        assert!(s.is_completing());
        rt.complete_action(agent);

        // Tick 2: chase (1 tick)
        let s = rt.tick(agent);
        assert_eq!(s.action_id(), Some("chase_target"));
        rt.complete_action(agent);

        // Tick 3..32: melee_attack (30 ticks)
        for i in 1..=30 {
            let s = rt.tick(agent);
            assert_eq!(s.action_id(), Some("melee_attack"));
            assert_eq!(
                s.is_completing(),
                i == 30,
                "completing should only be true on the 30th tick"
            );
        }
        rt.complete_action(agent);

        assert_eq!(rt.tick(agent), RuntimeStep::Idle);
    }
}
