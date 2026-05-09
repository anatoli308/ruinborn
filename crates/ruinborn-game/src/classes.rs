//! D2 character classes — Barbarian, Sorceress, Necromancer.
//!
//! Each class ships a base `Stats` block and a small starter skill list.
//! The actual skill effects live in [`crate::skills`].

use serde::{Deserialize, Serialize};

use crate::progression::Stats;

/// One of the three playable classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassId {
    Barbarian,
    Sorceress,
    Necromancer,
}

impl ClassId {
    pub fn label(self) -> &'static str {
        match self {
            ClassId::Barbarian => "Barbarian",
            ClassId::Sorceress => "Sorceress",
            ClassId::Necromancer => "Necromancer",
        }
    }

    /// Parse a snake_case identifier (e.g. from a JSON message).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "barbarian" => Some(ClassId::Barbarian),
            "sorceress" => Some(ClassId::Sorceress),
            "necromancer" => Some(ClassId::Necromancer),
            _ => None,
        }
    }
}

/// Static metadata about a class — base stats + the skill ids unlocked at level 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDefinition {
    pub id: ClassId,
    pub name: String,
    pub base_stats: Stats,
    pub starter_skills: Vec<String>,
}

/// Lookup the static class definition. Pure function — no game state involved.
pub fn class_definition(id: ClassId) -> ClassDefinition {
    match id {
        ClassId::Barbarian => ClassDefinition {
            id,
            name: "Barbarian".into(),
            base_stats: Stats { strength: 30, dexterity: 20, vitality: 25, energy: 10 },
            starter_skills: vec!["bash".into()],
        },
        ClassId::Sorceress => ClassDefinition {
            id,
            name: "Sorceress".into(),
            base_stats: Stats { strength: 10, dexterity: 25, vitality: 10, energy: 35 },
            starter_skills: vec!["fireball".into()],
        },
        ClassId::Necromancer => ClassDefinition {
            id,
            name: "Necromancer".into(),
            base_stats: Stats { strength: 15, dexterity: 25, vitality: 15, energy: 25 },
            starter_skills: vec!["bone_spear".into()],
        },
    }
}

/// All three classes — useful for the class-select screen.
pub fn all_classes() -> Vec<ClassDefinition> {
    vec![
        class_definition(ClassId::Barbarian),
        class_definition(ClassId::Sorceress),
        class_definition(ClassId::Necromancer),
    ]
}
