//! Agent configuration + JSON-driven registry.
//!
//! Mirrors [`crate::enemy_archetype`]: `OnceLock` storage, bundled JSON
//! via `include_str!`, optional runtime override at server startup.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::action::GoapAction;
use super::goal::Goal;
use super::world_state::WorldState;

/// Bundled defaults — recompiled into the binary so we always have a
/// fallback agent config for every archetype declared here.
const BUNDLED_JSON: &str = include_str!("../../../data/goap/agents.json");

/// Full GOAP config for one agent kind. The `id` matches an entry in
/// `enemies.json` so each `Enemy.kind` maps 1:1 to its planner setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoapAgentConfig {
    pub id: String,

    /// Initial world facts written before the agent's first sensor pass
    /// runs. Lets authors seed defaults like `"has_target": false`.
    #[serde(default)]
    pub initial_world: WorldState,

    pub goals: Vec<Goal>,
    pub actions: Vec<GoapAction>,
}

impl GoapAgentConfig {
    pub fn goal(&self, id: &str) -> Option<&Goal> {
        self.goals.iter().find(|g| g.id == id)
    }

    pub fn action(&self, id: &str) -> Option<&GoapAction> {
        self.actions.iter().find(|a| a.id == id)
    }
}

/// In-memory immutable registry. Built once, queried by archetype id.
#[derive(Debug, Default)]
pub struct GoapRegistry {
    agents: HashMap<String, GoapAgentConfig>,
}

impl GoapRegistry {
    pub fn agent(&self, id: &str) -> Option<&GoapAgentConfig> {
        self.agents.get(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.agents.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct AgentsFile {
    #[serde(default)]
    #[allow(dead_code)]
    _comment: serde_json::Value,
    agents: Vec<GoapAgentConfig>,
}

static REGISTRY: OnceLock<GoapRegistry> = OnceLock::new();

/// Parse and validate an agents file. Returns the registry on success
/// or a human-readable error describing the first problem found.
pub fn load_agents_from_str(json: &str) -> Result<GoapRegistry, String> {
    let file: AgentsFile = serde_json::from_str(json)
        .map_err(|e| format!("failed to parse goap agents.json: {e}"))?;

    let mut agents: HashMap<String, GoapAgentConfig> = HashMap::new();

    for agent in file.agents {
        validate_agent(&agent)?;
        if agents.insert(agent.id.clone(), agent.clone()).is_some() {
            return Err(format!("duplicate goap agent id: {}", agent.id));
        }
    }

    Ok(GoapRegistry { agents })
}

fn validate_agent(agent: &GoapAgentConfig) -> Result<(), String> {
    if agent.goals.is_empty() {
        return Err(format!("goap agent '{}' has no goals", agent.id));
    }
    if agent.actions.is_empty() {
        return Err(format!("goap agent '{}' has no actions", agent.id));
    }

    let mut goal_ids: HashSet<&str> = HashSet::new();
    for g in &agent.goals {
        if !goal_ids.insert(&g.id) {
            return Err(format!(
                "goap agent '{}' has duplicate goal id '{}'",
                agent.id, g.id
            ));
        }
    }

    let mut action_ids: HashSet<&str> = HashSet::new();
    for a in &agent.actions {
        if !action_ids.insert(&a.id) {
            return Err(format!(
                "goap agent '{}' has duplicate action id '{}'",
                agent.id, a.id
            ));
        }
        if a.behaviour.trim().is_empty() {
            return Err(format!(
                "goap action '{}/{}' has empty behaviour tag",
                agent.id, a.id
            ));
        }
    }

    Ok(())
}

/// Lazily initialise the global registry from the bundled JSON. Panics
/// if the bundled file is malformed — that's a build-time bug, not a
/// runtime condition.
pub fn registry() -> &'static GoapRegistry {
    REGISTRY.get_or_init(|| {
        load_agents_from_str(BUNDLED_JSON)
            .expect("bundled goap agents.json is malformed — fix data/goap/agents.json")
    })
}

/// Look up an agent config by archetype id. Returns `None` for kinds
/// that have no GOAP config (those keep their legacy hard-coded AI in
/// Phase 4 until you choose to migrate them).
pub fn try_agent_config(id: &str) -> Option<&'static GoapAgentConfig> {
    registry().agent(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_agents_parse() {
        let reg = load_agents_from_str(BUNDLED_JSON).expect("bundled file should parse");
        assert!(!reg.is_empty(), "expected at least one bundled agent");
    }

    #[test]
    fn duplicate_agent_id_rejected() {
        let json = r#"{
            "agents": [
                {"id": "a", "goals": [{"id":"g","label":"g","weight":1,"conditions":[]}],
                 "actions": [{"id":"x","label":"x","cost":1,"behaviour":"noop"}]},
                {"id": "a", "goals": [{"id":"g","label":"g","weight":1,"conditions":[]}],
                 "actions": [{"id":"x","label":"x","cost":1,"behaviour":"noop"}]}
            ]
        }"#;
        assert!(load_agents_from_str(json).is_err());
    }

    #[test]
    fn duplicate_goal_id_rejected() {
        let json = r#"{
            "agents": [{
                "id": "a",
                "goals": [
                    {"id":"g","label":"g","weight":1,"conditions":[]},
                    {"id":"g","label":"g","weight":1,"conditions":[]}
                ],
                "actions": [{"id":"x","label":"x","cost":1,"behaviour":"noop"}]
            }]
        }"#;
        assert!(load_agents_from_str(json).is_err());
    }

    #[test]
    fn empty_behaviour_rejected() {
        let json = r#"{
            "agents": [{
                "id": "a",
                "goals": [{"id":"g","label":"g","weight":1,"conditions":[]}],
                "actions": [{"id":"x","label":"x","cost":1,"behaviour":""}]
            }]
        }"#;
        assert!(load_agents_from_str(json).is_err());
    }
}
