//! Zone / Act system. D2-LoD-style: every zone has bounds, a kind, a spawn
//! point, and optionally a waypoint position the player can teleport to once
//! unlocked.
//!
//! ## Phase 6 — data-driven zones
//!
//! Zones are no longer a closed `enum`. [`ZoneId`] is a string-newtype
//! (`Arc<str>` inside) so that new areas can be added by editing
//! `crates/ruinborn-game/data/zones.json` without touching code.
//!
//! - `pub struct ZoneId(Arc<str>)` — `Clone`, `Hash`, `Eq`, `Serialize`,
//!   `Deserialize`. Serialises as the bare string id, so persisted player
//!   state keeps round-tripping cleanly.
//! - `Zone` carries an `act: u8` tag, a `neighbors: Vec<ZoneId>` adjacency
//!   list (used later for transition + waypoint validation), and the bounds
//!   / spawn / waypoint info as before.
//! - The default content is **all of D2 Act 1** — town, both wilderness
//!   spines, the Monastery dungeon chain ending at Catacombs L4, plus the
//!   Burial Grounds / Cave / Pit side dungeons. ~31 zones.
//!
//! Hot reload is intentionally *not* supported: zone bounds are baked into
//! enemy + player positions, so a live edit could leave entities outside
//! their zone. Zones load once at startup via [`build_default_zones`].

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::sync::OnceLock;

// ── ZoneId ────────────────────────────────────────────────────

/// Stable, serialisable zone identifier. Internally an `Arc<str>` so cloning
/// is cheap (single atomic increment) and equality is value-based.
///
/// Wire format is the bare snake_case string — e.g. `"rogue_encampment"`,
/// `"jail_l1"`. This means old persisted player state with `"town"` /
/// `"wilderness"` / `"burial_grounds"` round-trips through migration in
/// [`ZoneId::from_legacy`].
#[derive(Debug, Clone)]
pub struct ZoneId(Arc<str>);

impl ZoneId {
    /// Construct from any string-like value. Always succeeds.
    pub fn new<S: Into<Arc<str>>>(s: S) -> Self {
        Self(s.into())
    }

    /// Borrow the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Map deprecated zone ids onto their Phase-6 D2 equivalents. Used by
    /// the persistence layer to migrate Phase-1 saves transparently.
    pub fn from_legacy(s: &str) -> ZoneId {
        match s {
            "town" => ZoneId::new("rogue_encampment"),
            "wilderness" => ZoneId::new("blood_moor"),
            // burial_grounds + everything else passes through unchanged.
            other => ZoneId::new(other),
        }
    }
}

impl PartialEq for ZoneId {
    fn eq(&self, other: &Self) -> bool {
        // Pointer equality first (cheap fast path for cloned ids), value
        // equality otherwise — required because two zones loaded through
        // different code paths may end up with distinct Arcs.
        Arc::ptr_eq(&self.0, &other.0) || *self.0 == *other.0
    }
}
impl Eq for ZoneId {}

impl std::hash::Hash for ZoneId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the bytes, not the Arc — must agree with PartialEq.
        self.0.as_bytes().hash(state);
    }
}

impl fmt::Display for ZoneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ZoneId {
    fn from(s: &str) -> Self {
        ZoneId::new(s)
    }
}

impl From<String> for ZoneId {
    fn from(s: String) -> Self {
        ZoneId::new(s)
    }
}

impl Serialize for ZoneId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ZoneId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(ZoneId::new(s))
    }
}

// ── ZoneKind ─────────────────────────────────────────────────

/// What kind of zone this is — drives spawn-table selection (see
/// [`crate::enemy_archetype::pick_archetype_for_zone`]) and PvE rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    Town,
    Wilderness,
    Dungeon,
}

// ── Bounds ────────────────────────────────────────────────────

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
        (
            x.clamp(self.min_x, self.max_x),
            z.clamp(self.min_z, self.max_z),
        )
    }
    pub fn overlaps(&self, other: &ZoneBounds) -> bool {
        self.min_x < other.max_x
            && other.min_x < self.max_x
            && self.min_z < other.max_z
            && other.min_z < self.max_z
    }
}

// ── Zone ─────────────────────────────────────────────────────

/// Static zone definition. Lives in `GameState.zones`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: ZoneId,
    pub name: String,
    /// Which D2 act this zone belongs to (1..=5). Used as a grouping tag.
    #[serde(default = "default_act")]
    pub act: u8,
    pub kind: ZoneKind,
    pub bounds: ZoneBounds,
    /// Where new players (or waypoint-travelling players) appear.
    pub spawn_x: f64,
    pub spawn_z: f64,
    /// Optional waypoint position (teleport target).
    pub waypoint_x: Option<f64>,
    pub waypoint_z: Option<f64>,
    /// Target enemy population for this zone (0 = peaceful).
    pub enemy_target: u32,
    /// Adjacent zones reachable on foot (for waypoint validation, future
    /// transition triggers, and pathfinding overlays). Bidirectional by
    /// convention — keep both sides in sync when authoring.
    #[serde(default)]
    pub neighbors: Vec<ZoneId>,
}

fn default_act() -> u8 {
    1
}

// ── Zone catalogue (data-driven) ─────────────────────────────

/// Bundled JSON catalogue. Compiled into the binary so a fresh server
/// always boots with at least Act 1 available.
const ZONES_JSON: &str = include_str!("../data/zones.json");

#[derive(Deserialize)]
struct ZonesFile {
    zones: Vec<Zone>,
}

static ZONES: OnceLock<Vec<Zone>> = OnceLock::new();

/// Load and validate the zone catalogue once. Subsequent calls return the
/// cached value. Validation guarantees:
///
/// 1. At least one zone of [`ZoneKind::Town`] exists (for respawns).
/// 2. No two zones share the same [`ZoneId`].
/// 3. Every neighbour reference resolves to a known zone.
/// 4. Spawn point is inside the zone bounds.
///
/// Validation failure panics — this is start-up content, not user input,
/// so a corrupt file is a programmer error worth surfacing immediately.
pub fn zone_catalogue() -> &'static [Zone] {
    ZONES.get_or_init(load_and_validate).as_slice()
}

fn load_and_validate() -> Vec<Zone> {
    let file: ZonesFile = serde_json::from_str(ZONES_JSON)
        .expect("failed to parse bundled zones.json — fix the file");
    let zones = file.zones;

    let mut seen: HashSet<&str> = HashSet::new();
    for z in &zones {
        if !seen.insert(z.id.as_str()) {
            panic!("duplicate zone id `{}` in zones.json", z.id);
        }
        if !z.bounds.contains(z.spawn_x, z.spawn_z) {
            panic!(
                "zone `{}` has spawn ({:.1}, {:.1}) outside bounds",
                z.id, z.spawn_x, z.spawn_z
            );
        }
    }

    if !zones.iter().any(|z| z.kind == ZoneKind::Town) {
        panic!("zones.json must contain at least one zone of kind=town");
    }

    let ids: HashSet<&str> = zones.iter().map(|z| z.id.as_str()).collect();
    for z in &zones {
        for n in &z.neighbors {
            if !ids.contains(n.as_str()) {
                panic!(
                    "zone `{}` references unknown neighbour `{}`",
                    z.id, n
                );
            }
        }
    }

    zones
}

/// Snapshot of the zone catalogue for [`crate::market::GameState`] to own.
/// Returns owned `Vec<Zone>` (cheap — `Zone` clones share Arc string ids).
pub fn build_default_zones() -> Vec<Zone> {
    zone_catalogue().to_vec()
}

// ── Lookup helpers ───────────────────────────────────────────

/// Find which zone a world-position falls into. None if in dead space
/// (between bounds rectangles).
pub fn zone_at(zones: &[Zone], x: f64, z: f64) -> Option<ZoneId> {
    zones
        .iter()
        .find(|z0| z0.bounds.contains(x, z))
        .map(|z0| z0.id.clone())
}

pub fn zone_by_id<'a>(zones: &'a [Zone], id: &ZoneId) -> Option<&'a Zone> {
    zones.iter().find(|z| z.id == *id)
}

/// First zone of [`ZoneKind::Town`] in the catalogue. Used as the canonical
/// respawn target — replaces the old `ZoneId::Town` constant.
pub fn town_zone(zones: &[Zone]) -> &Zone {
    zones
        .iter()
        .find(|z| z.kind == ZoneKind::Town)
        .expect("zone catalogue invariant: at least one town zone")
}

/// Convenience: id of the canonical town zone.
pub fn town_zone_id(zones: &[Zone]) -> ZoneId {
    town_zone(zones).id.clone()
}

/// Grant a waypoint unlock. Returns true if it was newly unlocked.
pub fn unlock_waypoint(unlocked: &mut HashSet<ZoneId>, id: ZoneId) -> bool {
    unlocked.insert(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_loads_and_validates() {
        let zs = zone_catalogue();
        assert!(!zs.is_empty());
        assert!(zs.iter().any(|z| z.kind == ZoneKind::Town));
    }

    #[test]
    fn no_overlapping_bounds_within_act() {
        let zs = zone_catalogue();
        for (i, a) in zs.iter().enumerate() {
            for b in zs.iter().skip(i + 1) {
                if a.act == b.act && a.bounds.overlaps(&b.bounds) {
                    panic!(
                        "zones `{}` and `{}` (act {}) have overlapping bounds",
                        a.id, b.id, a.act
                    );
                }
            }
        }
    }

    #[test]
    fn town_lookup_works() {
        let zs = zone_catalogue();
        let town = town_zone(zs);
        assert_eq!(town.kind, ZoneKind::Town);
    }

    #[test]
    fn zone_id_round_trips_through_serde() {
        let z = ZoneId::new("rogue_encampment");
        let s = serde_json::to_string(&z).unwrap();
        assert_eq!(s, "\"rogue_encampment\"");
        let back: ZoneId = serde_json::from_str(&s).unwrap();
        assert_eq!(back, z);
    }

    #[test]
    fn zone_id_legacy_migration() {
        assert_eq!(
            ZoneId::from_legacy("town"),
            ZoneId::new("rogue_encampment")
        );
        assert_eq!(
            ZoneId::from_legacy("wilderness"),
            ZoneId::new("blood_moor")
        );
        assert_eq!(
            ZoneId::from_legacy("burial_grounds"),
            ZoneId::new("burial_grounds")
        );
        assert_eq!(
            ZoneId::from_legacy("dark_wood"),
            ZoneId::new("dark_wood")
        );
    }

    #[test]
    fn zone_at_finds_town_at_origin() {
        let zs = zone_catalogue();
        let town = town_zone(zs);
        let id = zone_at(zs, town.spawn_x, town.spawn_z);
        assert_eq!(id, Some(town.id.clone()));
    }

    #[test]
    fn neighbors_are_symmetric_for_act_1() {
        // Lint-style check: catch authoring drift where A says B is a
        // neighbour but B doesn't list A. Not a hard load-time invariant
        // since one-way portals (e.g. Tristram return-portal) are valid
        // in principle, but right now we don't author any.
        let zs = zone_catalogue();
        for a in zs {
            for n_id in &a.neighbors {
                let b = zone_by_id(zs, n_id).expect("neighbour exists");
                assert!(
                    b.neighbors.iter().any(|x| x == &a.id),
                    "asymmetric neighbour: `{}` -> `{}` but not back",
                    a.id,
                    b.id
                );
            }
        }
    }
}
