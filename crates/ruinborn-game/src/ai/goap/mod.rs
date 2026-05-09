//! GOAP — Goal-Oriented Action Planning.
//!
//! ## Phase 1 — Data model (this file set)
//!
//! Pure data structures plus a JSON-driven registry. No resolver, no
//! runtime. Lets us author agent configs and round-trip them through
//! `serde` so Phase 2 (the A* resolver) and Phase 3 (the per-tick agent
//! runtime) can be built and tested in isolation.
//!
//! ## Concepts (mirrors CrashKonijn GOAP terminology)
//!
//! - **WorldKey** — a named boolean/integer fact about the world
//!   (`"has_target"`, `"in_attack_range"`, `"hp_percent"`, …).
//! - **TargetKey** — a named *thing* the agent might act on
//!   (`"nearest_player"`, `"spawn_anchor"`).
//! - **SenseValue** — current value of a `WorldKey`, written by sensors.
//! - **WorldState** — full `WorldKey -> SenseValue` map for one agent.
//! - **Condition** — `(key, comparison, value)` predicate over the world.
//! - **Effect** — what an action *promises* to do to the world if it runs.
//! - **Goal** — a desired set of conditions; planner tries to satisfy.
//! - **Action** — has preconditions, effects, cost, and an optional target.
//! - **Agent config** — bundle of goals + actions + initial world state,
//!   keyed by archetype id (matches `enemies.json`).
//!
//! ## Authoring
//!
//! Edit [`crates/ruinborn-game/data/goap/agents.json`]. The bundled file
//! is embedded via `include_str!` so the binary always boots; the server
//! may call [`load_agents_from_str`] at startup to override.
//!
//! No Rust changes are needed for new goals, actions, or balance tweaks
//! — only when introducing a brand-new *behaviour kind* (the `behaviour`
//! string on an action) does Phase 3 need a new dispatch arm.

mod action;
mod condition;
mod config;
mod goal;
mod resolver;
mod runtime;
mod world_state;

pub use action::{GoapAction, TargetKey};
pub use condition::{Comparison, Condition, Effect, EffectOp};
pub use config::{
    load_agents_from_str, registry, try_agent_config, GoapAgentConfig, GoapRegistry,
};
pub use goal::Goal;
pub use resolver::{plan, Plan, PlanFailure, MAX_NODES};
pub use runtime::{GoapAgentRuntime, RuntimeStep};
pub use world_state::{SenseValue, WorldKey, WorldState};
