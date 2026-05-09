//! Zone / Act system. D2-LoD-style: every zone has bounds, a kind, a spawn point,
//! and optionally a waypoint position the player can teleport to once unlocked.
//!
//! Phase 1 layout:
//! - `Town`           — Act 1 city (Rogue Encampment-ish). Safe. No enemies. Waypoint at center.
//! - `Wilderness`     — Act 1 outdoors (Blood Moor / Cold Plains). Spawns weak enemies. Has waypoint.
//! - `BurialGrounds`  — Act 1 dungeon-zone (Burial Grounds). Tougher enemies, has waypoint.
//!
//! All three zones share the same flat 2D ground plane; transition is by walking
//! across the zone border. The whole world is one big map, the zones partition it.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Stable, serializable zone identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneId {
    Town,
    Wilderness,
    BurialGrounds,
}

impl ZoneId {
    pub fn all() -> [ZoneId; 3] {
        [ZoneId::Town, ZoneId::Wilderness, ZoneId::BurialGrounds]
    }
    pub fn label(self) -> &'static str {
        match self {
            ZoneId::Town => "Stadt",
            ZoneId::Wilderness => "Wildnis",
            ZoneId::BurialGrounds => "Gr\u{00E4}berfeld",
        }
    }
}

/// What kind of zone this is — controls spawning and PvE rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    Town,
    Wilderness,
    Dungeon,
}

/// Axis-aligned rectangle in world space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ZoneBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl ZoneBounds {
    pub fn contains(&self, x: f64, z: f64) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }
    pub fn clamp(&self, x: f64, z: f64) -> (f64, f64) {
        (x.clamp(self.min_x, self.max_x), z.clamp(self.min_z, self.max_z))
    }
}

/// Static zone definition. Lives in `GameState.zones`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: ZoneId,
    pub name: String,
    pub kind: ZoneKind,
    pub bounds: ZoneBounds,
    /// Where new players (or waypoint-travelling players) appear.
    pub spawn_x: f64,
    pub spawn_z: f64,
    /// Optional waypoint position (teleport target). Same coords as spawn for MVP.
    pub waypoint_x: Option<f64>,
    pub waypoint_z: Option<f64>,
    /// Target enemy population for this zone (0 = peaceful).
    pub enemy_target: u32,
}

/// Build the Phase-1 zone layout.
pub fn build_default_zones() -> Vec<Zone> {
    // Town: -30..30 x -30..30 (60x60 safe square), spawn center.
    // Wilderness: stretch to the south (negative z): -60..60 x  30..90.
    // Burial Grounds: stretch east of wilderness:    60..120 x 30..120.
    vec![
        Zone {
            id: ZoneId::Town,
            name: "Stadt".into(),
            kind: ZoneKind::Town,
            bounds: ZoneBounds { min_x: -30.0, max_x: 30.0, min_z: -30.0, max_z: 30.0 },
            spawn_x: 0.0,
            spawn_z: 0.0,
            waypoint_x: Some(0.0),
            waypoint_z: Some(0.0),
            enemy_target: 0,
        },
        Zone {
            id: ZoneId::Wilderness,
            name: "Wildnis (Akt 1)".into(),
            kind: ZoneKind::Wilderness,
            bounds: ZoneBounds { min_x: -60.0, max_x: 60.0, min_z: 30.0, max_z: 90.0 },
            spawn_x: 0.0,
            spawn_z: 35.0,
            waypoint_x: Some(0.0),
            waypoint_z: Some(60.0),
            enemy_target: 8,
        },
        Zone {
            id: ZoneId::BurialGrounds,
            name: "Gr\u{00E4}berfeld".into(),
            kind: ZoneKind::Dungeon,
            bounds: ZoneBounds { min_x: 60.0, max_x: 120.0, min_z: 30.0, max_z: 120.0 },
            spawn_x: 65.0,
            spawn_z: 35.0,
            waypoint_x: Some(90.0),
            waypoint_z: Some(75.0),
            enemy_target: 6,
        },
    ]
}

/// Find which zone (by id) a world-position falls into. None if in dead space.
pub fn zone_at(zones: &[Zone], x: f64, z: f64) -> Option<ZoneId> {
    zones.iter().find(|z0| z0.bounds.contains(x, z)).map(|z0| z0.id)
}

pub fn zone_by_id(zones: &[Zone], id: ZoneId) -> Option<&Zone> {
    zones.iter().find(|z| z.id == id)
}

/// Grant a waypoint unlock. Returns true if it was newly unlocked.
pub fn unlock_waypoint(unlocked: &mut HashSet<ZoneId>, id: ZoneId) -> bool {
    unlocked.insert(id)
}
