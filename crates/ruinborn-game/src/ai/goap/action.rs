//! Actions — what an agent can do to change the world.

use serde::{Deserialize, Serialize};

use super::condition::{Condition, Effect};

/// Stable identifier for a *thing* an action operates on, resolved at
/// runtime by a target sensor (e.g. `"nearest_player"`,
/// `"spawn_anchor"`). String-typed for the same authoring reason as
/// [`super::WorldKey`].
pub type TargetKey = String;

/// One action in an agent's repertoire. The planner chains these
/// backwards from a goal until all preconditions reduce to the current
/// world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoapAction {
    pub id: String,
    pub label: String,

    /// Static planning cost. Lower = preferred. The runtime may add a
    /// dynamic component (distance, energy, …) on top later.
    pub cost: f64,

    /// All must hold for the action to be considered runnable.
    #[serde(default)]
    pub preconditions: Vec<Condition>,

    /// Applied to the simulated world during planning, and to the real
    /// world after the action runs.
    #[serde(default)]
    pub effects: Vec<Effect>,

    /// If `Some`, the runtime resolves this target via a target sensor
    /// and considers the action runnable only when the agent is within
    /// [`in_range`] world units of that target.
    #[serde(default)]
    pub target_key: Option<TargetKey>,

    /// World-unit radius around the target within which the action can
    /// execute. Ignored if `target_key` is `None`.
    #[serde(default)]
    pub in_range: f64,

    /// How many combat ticks the action takes to complete once started.
    /// `1` = instant on next tick.
    #[serde(default = "default_ticks_to_perform")]
    pub ticks_to_perform: u32,

    /// Dispatch tag used by the Phase 3 runtime to pick the concrete
    /// Rust handler (e.g. `"move_to_target"`, `"melee_attack"`,
    /// `"wander"`, `"flee"`). The planner never inspects this — it's
    /// purely a runtime hook.
    pub behaviour: String,
}

fn default_ticks_to_perform() -> u32 {
    1
}
