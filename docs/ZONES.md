# Zonen — Datengetrieben (Phase 6)

Stand: D2 LoD Act 1 vollständig modelliert (31 Zonen, 9 Wegpunkte).
Alle Zonen leben in `crates/ruinborn-game/data/zones.json`. Code in
`crates/ruinborn-game/src/world.rs`.

---

## 1 · `ZoneId` Newtype

```rust
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ZoneId(Arc<str>);
```

- **Newtype über `Arc<str>`** — kein `Copy`, aber günstiges `Clone`
  (RefCount-Inkrement, kein String-Realloc).
- `From<&str>`, `From<String>`, `as_str()`, `Display`.
- **Migration**: `ZoneId::from_legacy(s: &str) -> ZoneId` mappt alte
  Save-Daten auf neue Ids:

  | Legacy String     | Neue Id            |
  | ----------------- | ------------------ |
  | `"town"`          | `rogue_encampment` |
  | `"wilderness"`    | `blood_moor`       |
  | `"burial_grounds"`| `burial_grounds` (passthrough) |
  | sonst             | passthrough        |

- DB-Spalte `players.zone` bleibt `TEXT` — Migration ist transparent.

---

## 2 · `Zone` Struct & JSON-Schema

```rust
pub struct Zone {
    pub id: ZoneId,
    pub name: String,
    pub act: u8,
    pub kind: ZoneKind,         // Town | Wilderness | Dungeon
    pub bounds: ZoneBounds,     // axis-aligned rectangle
    pub spawn_x: f32, pub spawn_z: f32,
    pub waypoint_x: Option<f32>, pub waypoint_z: Option<f32>,
    pub enemy_target: u32,      // Soll-Population für Spawner
    pub neighbors: Vec<ZoneId>,
}
```

JSON-Felder 1:1 (snake_case):

```jsonc
{
  "id": "cold_plains",
  "name": "Cold Plains",
  "act": 1,
  "kind": "wilderness",
  "bounds": { "min_x": -30.0, "max_x": 30.0, "min_z": 90.0, "max_z": 150.0 },
  "spawn_x": 0.0, "spawn_z": 95.0,
  "waypoint_x": 0.0, "waypoint_z": 120.0,
  "enemy_target": 30,
  "neighbors": ["blood_moor", "cave_l1", "stony_field", "burial_grounds"]
}
```

Layout-Konvention: 60×60-Welt-Einheiten pro Zone, gekachelt entlang +Z (Norden) ab Town,
mit Ost-/West-Abzweigen für Sub-Dungeons. Town spawnt auf `(0,0)`.

---

## 3 · Loader & Validierung

`world.rs::load_and_validate()` läuft beim `GameState::new()`:

1. Parse `include_str!("../data/zones.json")` → `Vec<ZoneRaw>`.
2. Für jedes Raw → `Zone` builden, in `HashMap<ZoneId, Zone>` einfügen.
3. **Invarianten** (panic bei Verstoß — fail fast):
   - Keine Duplikat-Ids.
   - Genau eine Zone mit `kind == Town`.
   - Spawn-Punkt liegt innerhalb `bounds`.
   - Jeder Eintrag in `neighbors` zeigt auf existierende Zone.
   - Nachbarschaft ist symmetrisch (`A in B.neighbors ⇔ B in A.neighbors`).

Catalogue wird in einem `OnceLock<ZoneCatalogue>` gehalten; gleichzeitig
liegt `Arc<Vec<Zone>>` im `GameState.zones` für serialisierbaren Zugriff.

---

## 4 · D2 Act 1 — vollständige Zonenliste

**31 Zonen, 9 Wegpunkte**.

| # | Id                       | Kind      | Wegpunkt | Nachbarn |
|---|--------------------------|-----------|----------|----------|
| 1 | `rogue_encampment`       | town      | ✓        | blood_moor |
| 2 | `blood_moor`             | wilderness|          | rogue_encampment, den_of_evil, cold_plains |
| 3 | `den_of_evil`            | dungeon   |          | blood_moor |
| 4 | `cold_plains`            | wilderness| ✓        | blood_moor, cave_l1, stony_field, burial_grounds |
| 5 | `cave_l1`                | dungeon   |          | cold_plains, cave_l2 |
| 6 | `cave_l2`                | dungeon   |          | cave_l1 |
| 7 | `burial_grounds`         | wilderness|          | cold_plains, crypt, mausoleum |
| 8 | `crypt`                  | dungeon   |          | burial_grounds |
| 9 | `mausoleum`              | dungeon   |          | burial_grounds |
| 10 | `stony_field`           | wilderness| ✓        | cold_plains, tristram, underground_passage_l1 |
| 11 | `tristram`              | dungeon   |          | stony_field |
| 12 | `underground_passage_l1`| dungeon   |          | stony_field, underground_passage_l2 |
| 13 | `underground_passage_l2`| dungeon   |          | underground_passage_l1, dark_wood |
| 14 | `dark_wood`             | wilderness| ✓        | underground_passage_l2, black_marsh |
| 15 | `black_marsh`           | wilderness| ✓        | dark_wood, forgotten_tower, tamoe_highland |
| 16 | `forgotten_tower`       | dungeon   |          | black_marsh |
| 17 | `tamoe_highland`        | wilderness|          | black_marsh, the_pit_l1, monastery_gate |
| 18 | `the_pit_l1`            | dungeon   |          | tamoe_highland, the_pit_l2 |
| 19 | `the_pit_l2`            | dungeon   |          | the_pit_l1 |
| 20 | `monastery_gate`        | wilderness|          | tamoe_highland, outer_cloister |
| 21 | `outer_cloister`        | dungeon   | ✓        | monastery_gate, barracks |
| 22 | `barracks`              | dungeon   |          | outer_cloister, jail_l1 |
| 23 | `jail_l1`               | dungeon   | ✓        | barracks, jail_l2 |
| 24 | `jail_l2`               | dungeon   |          | jail_l1, jail_l3 |
| 25 | `jail_l3`               | dungeon   |          | jail_l2, inner_cloister |
| 26 | `inner_cloister`        | dungeon   | ✓        | jail_l3, cathedral |
| 27 | `cathedral`             | dungeon   |          | inner_cloister, catacombs_l1 |
| 28 | `catacombs_l1`          | dungeon   |          | cathedral, catacombs_l2 |
| 29 | `catacombs_l2`          | dungeon   | ✓        | catacombs_l1, catacombs_l3 |
| 30 | `catacombs_l3`          | dungeon   |          | catacombs_l2, catacombs_l4 |
| 31 | `catacombs_l4`          | dungeon   |          | catacombs_l3 |

---

## 5 · Wegpunkt-Travel

- Server prüft `zone.waypoint_x.is_some()`, ob Travel erlaubt ist.
- Client schickt `ClientMessage::TravelWaypoint { zone: String }` →
  Server konvertiert via `ZoneId::from(zone.as_str())`.
- Spieler poppt am `(spawn_x, spawn_z)` der Zielzone, neue `zone` wird
  in `players.zone` persistiert.
- UI: `WaypointTravel.tsx` listet alle Zonen mit `waypoint_x.is_some()`.

---

## 6 · Enemy-Spawning

`combat::maintain_population` durchläuft alle Zonen:

```
für jede zone:
    aktuelle = Anzahl Enemies mit enemy.zone == zone.id
    while aktuelle < zone.enemy_target:
        archetype = enemy_archetype::pick_archetype_for_zone(zone.kind)
        spawn_pack(zone, archetype)
        aktuelle += pack_size
```

`enemy_archetype.rs` hält Spawn-Tabellen pro `ZoneKind`. Town hat
`enemy_target = 0` → nie Mobs.

---

## 7 · Erweitern

Neue Zone hinzufügen:

1. Eintrag in `data/zones.json` anhängen.
2. `neighbors` symmetrisch in beiden beteiligten Zonen pflegen.
3. Optional: Wegpunkt setzen (`waypoint_x` / `waypoint_z`).
4. `cargo test -p ruinborn-game` — Validator prüft Symmetrie + Spawn-Bounds.
5. Server neu starten (Catalogue ist `include_str!` — Rebuild nötig).

Für ein neues Akt-Set einfach `act: 2` setzen. Die UI gruppiert
`WaypointTravel` automatisch nach `act`.

---

## 8 · Tests

`crates/ruinborn-game/tests/zone_tests.rs` (auch in `world.rs`-Inline):

- Catalogue lädt ohne Panic.
- Genau 31 Zonen, genau 1 Town, 9 Wegpunkte.
- Alle `neighbors` sind symmetrisch.
- Alle Spawn-Punkte liegen in `bounds`.
- `ZoneId::from_legacy("town")` → `rogue_encampment`.
- `ZoneId::from_legacy("wilderness")` → `blood_moor`.

---

## 9 · Roadmap

- Zone-spezifische Loot-Tabellen (`loot_table_id` pro Zone).
- Boss-Zonen mit Unique-Spawn-Config (Andariel im Catacombs L4).
- Kachel-/Cell-Streaming für große Outdoor-Zonen (siehe
  [REFERENCE_VS_SOURCE.md](REFERENCE_VS_SOURCE.md) — `MapCellT`).
- Hot-Reload während Dev-Sessions.
