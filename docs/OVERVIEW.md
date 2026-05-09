# Ruinborn — Gesamt-Setup & Umsetzungen

Stand: Phase 6 (Data-Driven Zones — full D2 Act 1) abgeschlossen.
`cargo check --workspace` ✅ · `cargo test -p ruinborn-game` 47/47 ✅ · `cargo check -p ruinborn-server` ✅

Dieses Dokument fasst **das gesamte aktuelle Setup** zusammen — Architektur,
Module, Datenflüsse, Persistenz, UI und alle bisher umgesetzten Features.

> Begleitende Detail-Dokumente:
> - [ARCHITECTURE.md](ARCHITECTURE.md) — Kern-Architektur
> - [NETWORKING.md](NETWORKING.md) — Tauri/WS Protokoll
> - [DAMAGE_MODEL.md](DAMAGE_MODEL.md) — Phase 1–3 Klassen/Skills/Damage
> - [AI.md](AI.md) — Phase 4–5 GOAP-Planner + Boids-Steering
> - [ZONES.md](ZONES.md) — Phase 6 datengetriebene Zonen, Act 1 Graph
> - [REFERENCE_VS_SOURCE.md](REFERENCE_VS_SOURCE.md) — Vergleich C++ Referenz
> - [IDEEN.md](IDEEN.md) — geplante Features

---

## 1. Tech-Stack

| Schicht         | Tech                                                       |
| --------------- | ---------------------------------------------------------- |
| Desktop-Shell   | Tauri 2 (Rust)                                             |
| Backend / Game  | Rust — `ruinborn-game`, `ruinborn-protocol`, `ruinborn-server` |
| Persistenz      | PostgreSQL via SeaORM                                      |
| Async-Runtime   | Tokio                                                      |
| Frontend        | React 19 + TypeScript + Vite                               |
| 3D              | Three.js via React Three Fiber + Drei                      |
| State (Client)  | Zustand (Read-Only Mirror)                                 |
| Styling         | Tailwind CSS + PostCSS                                     |
| Transport       | WebSocket (Server → Tauri-Client)                          |

---

## 2. Workspace-Struktur

```
ruinborn/
├── Cargo.toml                  # Workspace root
├── package.json                # Frontend + Tauri scripts
├── crates/
│   ├── ruinborn-game/         # Pure logic, no I/O
│   │   ├── data/
│   │   │   ├── zones.json      # Phase 6: 31 Zonen, D2 Act 1
│   │   │   ├── enemies.json    # Archetype-Registry
│   │   │   └── goap/agents.json
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── classes.rs      # ClassId, ClassDef
│   │       ├── combat.rs       # Enemy, EnemyKind, attacks, ticks
│   │       ├── damage.rs       # DamageType/Tag, Resistances, DotInstance
│   │       ├── enemy_archetype.rs # JSON-Archetype-Registry, AiKind
│   │       ├── items.rs        # Items, Rarity, Affix, Equipment
│   │       ├── market.rs       # GameState, PlayerState, advance_tick
│   │       ├── progression.rs  # Level/XP-Curve, Stat-Punkte
│   │       ├── skills.rs       # SkillDef-Catalog, cast_skill
│   │       ├── world.rs        # ZoneId(Arc<str>), Zone, Wegpunkt-Graph
│   │       └── ai/
│   │           ├── boids.rs    # Phase 5: Reynolds-Flocking-Steering
│   │           └── goap/       # Phase 4: A*-Planner + Runtime
│   ├── ruinborn-protocol/     # WS message types
│   │   └── src/lib.rs          # ClientMessage / ServerMessage
│   └── ruinborn-server/       # WS-Server + DB
│       └── src/
│           ├── main.rs         # Tokio loop, tick-broadcast
│           ├── db.rs           # DB-Pool wiring
│           ├── db_sea.rs       # Schema-Migration + Load/Save
│           └── entity/         # SeaORM Models (player, market, …)
├── src-tauri/                  # Tauri shell (lädt Frontend, embed)
│   ├── src/{main,lib}.rs
│   ├── tauri.conf.json
│   └── capabilities/
├── src/                        # React Frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── ui/                 # HUD, Inventory, SkillTree, …
│   │   └── world/              # 3D-Szenen-Komponenten
│   ├── data/classes.ts         # SKILL_CATALOG, CLASS_CATALOG (Mirror)
│   ├── services/wsTransport.ts # WebSocket-Client
│   ├── store/gameStore.ts      # Zustand Store + Mappers
│   ├── types/index.ts          # Frontend Types (1:1 Server-Mirror)
│   └── styles/index.css
├── docs/
└── scripts/                    # Python helpers (write_market, write_store)
```

---

## 3. Architektur-Prinzipien

### Server-Authoritativ

- Aller Game-State lebt im Server-Prozess (`ruinborn-server`) als
  `Mutex<GameState>`.
- Frontend ist **Read-Only Mirror** — kein lokaler Mutationspfad.
- Spieler-Inputs gehen über WebSocket (`ClientMessage`), Antworten kommen als
  `ServerMessage::State` (Snapshot) oder `ServerMessage::Delta` (Patch).

### Single Source of Truth

- `ruinborn-game` enthält die einzige Implementierung jeder Spielregel.
- `ruinborn-protocol` definiert die einzige Wire-Format-Wahrheit.
- Frontend-Typen und -Catalogs sind *gespiegelt*, nicht *eigenständig*.

### Klare Trennungen

| Schicht          | Verantwortung                                  |
| ---------------- | ---------------------------------------------- |
| `ruinborn-game` | Pure Logik — kein I/O, keine DB, kein WS       |
| `ruinborn-server` | Tokio-Loop, WS-Handler, DB-Persistenz        |
| `ruinborn-protocol` | Nur Message-Schemas                        |
| Frontend Store   | Mapping snake_case → camelCase, IPC-Wrapper    |
| Frontend Components | Reines Rendering / Eingabe                  |

---

## 4. Tick-System

```
NETWORK_TICK_MS         = 50    // 20 Hz Broadcast
ECONOMY_TICK_INTERVAL   = 20    // ⇒ Game-Tick alle 1 s (5 Hz Game-Logik … nein: 1 Hz)
DB_SAVE_INTERVAL        = 30    // alle 30 Game-Ticks (= 30 s) speichern
```

Pro Network-Tick:
1. Spieler-Inputs aus `mpsc`-Channels einlesen, in `GameState` anwenden.
2. Bei Counter-Match: `ruinborn_game::market::advance_tick(&mut game)` — das
   **eine** Funktion, die Combat, Spawns, DoTs, Markt, Level-ups bewegt.
3. Pro verbundenem Client: `build_delta_snapshot(...)` und broadcast.
4. Periodisch: `db_sea::save_game_state(&pool, &game)`.

`advance_tick` (in `market.rs`) ruft sequentiell:
- `tick_enemies` → `Vec<EnemyHit>` einsammeln
- HP-Hits + ggf. Poison-DoT auf Spieler anwenden
- `tick_dots(&mut player.dots, &player.resistances)` — Tod-Check
- `tick_dots` pro Enemy
- Skill-Cooldowns / Buffs herunterzählen
- Markt-Preise / Resource-Spawns / Mission-Board ticken

---

## 5. Wire-Protocol (Auszug)

`ruinborn-protocol::ClientMessage` (`#[serde(tag = "cmd")]`):

| `cmd`                | Felder                                                                      |
| -------------------- | --------------------------------------------------------------------------- |
| `join`               | `name`                                                                      |
| `move`               | `dx, dz`                                                                    |
| `gather`             | —                                                                           |
| `create_market`      | `name`                                                                      |
| `post_order`         | `commodity_id, order_type, quantity, price_per_unit`                        |
| `cancel_order`       | `order_id`                                                                  |
| `fill_order`         | `market_id, order_id, quantity`                                             |
| `accept_mission`     | `mission_id`                                                                |
| `toggle_trade_panel` / `close_trade_panel` | —                                                     |
| `move_item`          | `src_bag, src_slot, dst_bag, dst_slot`                                      |
| `drop_item`          | `bag, slot`                                                                 |
| `set_action_slot`    | `slot, item_id?`                                                            |
| `use_action_slot`    | `slot`                                                                      |
| `equip_item`         | `bag, slot, target?`                                                        |
| `unequip_item`       | `target`                                                                    |
| `attack`             | `enemy_id, mouse_button`                                                    |
| `pickup_loot`        | `loot_id`                                                                   |
| `travel_waypoint`    | `zone`                                                                      |
| `allocate_stat`      | `stat`                                                                      |
| `set_mouse_skill`    | `mouse_button, item_id?`                                                    |
| `choose_class`       | `class`                                                                     |
| `allocate_skill`     | `skill_id`                                                                  |
| `cast_skill`         | `skill_id, target_enemy_id?, target_x?, target_z?`                          |

`ServerMessage` (`#[serde(tag = "type")]`): `state` · `delta` · `action_result`
· `welcome` · `error`.

---

## 6. Datenmodell — Rust

### `GameState` (Server)

```rust
pub struct GameState {
    pub tick: u64,
    pub elapsed_secs: f64,
    pub players: HashMap<String, PlayerState>,
    pub player_markets: Vec<PlayerMarket>,
    pub commodities: Vec<Commodity>,
    pub resource_nodes: Vec<ResourceNode>,
    pub enemies: Vec<Enemy>,
    pub loot_drops: Vec<LootDrop>,
    pub zones: Vec<Zone>,
    pub mission_board: Vec<Mission>,
}
```

### `PlayerState`

| Feld                        | Typ                              | Anmerkung                |
| --------------------------- | -------------------------------- | ------------------------ |
| `id, name`                  | `String`                         |                          |
| `x, z`                      | `f64`                            | Welt-Position            |
| `gold, reputation`          | `f64, u32`                       |                          |
| `inventory`                 | `Vec<ItemSlot>`                  | Legacy-Inventar          |
| `bags, action_bar, equipment` | siehe Items                    | D2-Paperdoll             |
| `level, xp, xp_to_next`     | `u32, u64, u64`                  |                          |
| `unspent_stat_points, stats` | `u32, Stats`                    | Str/Dex/Vit/Eng          |
| `hp, max_hp, mana, max_mana` | `f64`                            |                          |
| `is_dead, respawn_in`       | `bool, u32`                      |                          |
| `zone, unlocked_waypoints`  | `ZoneId, Vec<ZoneId>`            |                          |
| `mouse_left, mouse_right`   | `Option<ActionBinding>`          | LMB/RMB Bindings         |
| `class_id`                  | `Option<ClassId>`                | Phase 1                  |
| `allocated_skills`          | `HashMap<String, u32>`           |                          |
| `unspent_skill_points`      | `u32`                            |                          |
| `skill_cooldowns`           | `HashMap<String, u32>`           |                          |
| `active_buffs`              | `HashMap<String, u32>`           |                          |
| **`resistances`**           | `Resistances`                    | **Phase 3**              |
| **`dots`**                  | `Vec<DotInstance>`               | **Phase 3**              |
| `active_missions, trade_history` | `Vec<...>`                  |                          |
| `owned_market_id, nearest_market_id` | `Option<String>`         |                          |
| `notification`              | `String`                         | Toast-Meldungen          |

### Items (`items.rs`)

- **Rarity**: Common · Magic · Rare · Epic · Legendary (gewichtet, Crit-Rolls).
- **Affixes**: bis zu N Stat-Modifier pro Rarity-Stufe.
- **EquipSlotName**: Helmet, Amulet, Chest, Belt, Gloves, Boots, Weapon,
  Offhand, Ring1, Ring2 (D2-Paperdoll).
- **Bags**: 5 Slots (Default-Backpack + 4 wechselbare Bags).
- **ActionBar**: 9 Hotkey-Slots (`1`–`9`).

### Combat (`combat.rs`)

- **EnemyKind**: Zombie · Skeleton · FallenOne — pro Kind:
  - `attack_damage_type()` (Phase 3)
  - `poison_on_hit()` (Phase 3, nur Zombie)
  - `resistances()` (Phase 3)
- **`Enemy`**: id, kind, x/z, hp/max_hp, damage, state, target_player_id,
  `resistances`, `dots`, `despawn_in`.
- **`EnemyHit`** (Phase 3): `{ player_id, damage: DamageInstance, poison_dot? }`
- **`PlayerAttackOutcome`**: damage, damage_type, killed, xp_gained.

### Damage (`damage.rs`, **Phase 3**)

```rust
pub enum DamageType { Physical, Fire, Cold, Lightning, Poison, Magical }
pub enum DamageTag  { Melee, Ranged, Spell, Summoning, Trap }

pub struct DamageInstance {
    pub amount: f64,
    pub damage_type: DamageType,
    pub tags: Vec<DamageTag>,
}

pub struct Resistances {
    pub physical: f64, pub fire: f64, pub cold: f64,
    pub lightning: f64, pub poison: f64, pub magical: f64,
}
// Clamp: MAX_RESIST = 75.0
// apply(&DamageInstance) -> f64  // post-resist amount

pub struct DotInstance {
    pub damage_type: DamageType,
    pub damage_per_tick: f64,
    pub ticks_remaining: u32,
    pub tags: Vec<DamageTag>,
}
pub fn tick_dots(dots: &mut Vec<DotInstance>, &Resistances) -> f64
```

### Skills (`skills.rs`)

```rust
pub struct SkillDef {
    pub id: &'static str,
    pub name: &'static str,
    pub class_id: ClassId,
    pub mana_cost: f64,
    pub cooldown_ticks: u32,
    pub range: f64,
    pub requires_level: u32,
    pub effect: SkillEffect,
    pub damage_type: Option<DamageType>,   // Phase 3
    pub tags: &'static [DamageTag],         // Phase 3
}

pub enum SkillEffect {
    DirectDamage      { base: f64 },
    AoeAround         { base: f64, radius: f64 },
    DamageOverTime    { dps: f64, ticks: u32 },   // Phase 3
    Teleport,
    SelfBuff          { ticks: u32 },
    Placeholder,
}
```

| Skill              | Klasse      | Effect             | DmgType   | Tags             |
| ------------------ | ----------- | ------------------ | --------- | ---------------- |
| `bash`             | Barbar      | DirectDamage       | Physical  | [Melee]          |
| `cleave`           | Barbar      | AoeAround          | Physical  | [Melee]          |
| `battle_cry`       | Barbar      | SelfBuff           | —         | []               |
| `fireball`         | Zauberin    | DirectDamage       | Fire      | [Spell, Ranged]  |
| `frost_nova`       | Zauberin    | AoeAround          | Cold      | [Spell]          |
| `teleport`         | Zauberin    | Teleport           | —         | [Spell]          |
| `bone_spear`       | Necromancer | DirectDamage       | Magical   | [Spell, Ranged]  |
| `raise_skeleton`   | Necromancer | Placeholder        | —         | [Summoning]      |
| `amplify_damage`   | Necromancer | DamageOverTime     | Poison    | [Spell, Trap]    |

### World (`world.rs`)

- **`ZoneId`** (Phase 6): `pub struct ZoneId(Arc<str>)` — string-newtype,
  `Clone/Hash/Eq/Serialize/Deserialize`. Persistierte Saves migrieren via
  `ZoneId::from_legacy("town" | "wilderness" | "burial_grounds")`.
- **`Zone`**: `{ id, name, act: u8, kind: ZoneKind, bounds, spawn, waypoint?,
  enemy_target, neighbors: Vec<ZoneId> }`.
- **`ZoneKind`**: `Town`, `Wilderness`, `Dungeon` — steuert Spawn-Tabellen via
  `enemy_archetype::pick_archetype_for_zone`.
- **Catalogue**: `data/zones.json` (via `include_str!` gebundelt) → 31 Zonen,
  voller D2-LoD Act-1-Graph mit symmetrischen `neighbors`. Boot-Validierung
  panic-t auf duplicate ids, dangling neighbours, fehlende Town-Zone.
- **`WORLD_BOUND`** als globale Welt-Begrenzung (Backstop).
- Trading-Posts mit `interaction_range`-Constants.
- Details: siehe [ZONES.md](ZONES.md).

### Progression (`progression.rs`)

- XP-Kurve pro Level.
- Bei Level-Up: +1 Stat-Punkt, +1 Skill-Punkt, max-HP/Mana-Reskaling.

---

## 7. Persistenz — `players` (Postgres)

Schema wird per `db_sea::ensure_schema` **idempotent** mit `ALTER TABLE … ADD
COLUMN IF NOT EXISTS …` migriert. JSONB-Spalten erlauben Erweiterung ohne
Schema-Drift.

| Spalte                    | Typ          | Default       | Phase |
| ------------------------- | ------------ | ------------- | ----- |
| `id, name`                | TEXT         | —             | 0     |
| `x, z, gold`              | DOUBLE PRECISION | 0         | 0     |
| `inventory, bags`         | JSONB        | `[]/{}`       | 0     |
| `action_bar, equipment`   | JSONB        | `{}`          | 0     |
| `reputation, level, xp, xp_to_next` | INT/BIGINT | 0/1/0/100 | 0   |
| `unspent_stat_points`     | INT          | 0             | 0     |
| `stats`                   | JSONB        | `{}`          | 0     |
| `hp, max_hp, mana, max_mana` | FLOAT     | —             | 0     |
| `zone`                    | TEXT         | `'town'`      | 0     |
| `unlocked_waypoints`      | JSONB        | `[]`          | 0     |
| `mouse_left, mouse_right` | JSONB        | NULL          | 0     |
| `owned_market_id`         | TEXT         | NULL          | 0     |
| `class_id`                | TEXT         | NULL          | **1** |
| `allocated_skills`        | JSONB        | `{}`          | **1** |
| `unspent_skill_points`    | INT          | 0             | **1** |
| `skill_cooldowns`         | JSONB        | `{}`          | **1** |
| `active_buffs`            | JSONB        | `{}`          | **1** |
| **`resistances`**         | JSONB        | `{}`          | **3** |
| **`dots`**                | JSONB        | `[]`          | **3** |

Weitere Tabellen: `game_meta`, `player_market`, `market_order`,
`mission_board`, `player_mission`, `trade_history`, `resource_node`.

Load-Pattern: `serde_json::from_value(pm.<col>).unwrap_or_default()` →
backwards-kompatibel mit alten Spielständen.

---

## 8. Frontend

### Store-Mirror (`src/store/gameStore.ts`)

`ServerPlayer` hat alle snake_case-Felder vom Server (inkl.
`resistances`/`dots` aus Phase 3).
`GameStore` ist die camelCase-Variante mit zusätzlichen UI-Flags.

`mapSnapshot` und `mapDelta` mit Safe-Defaults überall:

```ts
resistances: s.player.resistances ?? { physical: 0, fire: 0, cold: 0,
                                       lightning: 0, poison: 0, magical: 0 },
dots: s.player.dots ?? [],
```

### UI-Komponenten

| Komponente                 | Funktion                                      |
| -------------------------- | --------------------------------------------- |
| `HUD.tsx`                  | HP/Mana-Globe, Level, Gold, Hotkey-Hints (inkl. `K Skills`) |
| `Inventory.tsx` / `InventoryWindow.tsx` | 5 Bags, Drag-and-Drop          |
| `BagBar.tsx`               | Bag-Wechsler                                  |
| `ActionBar.tsx`            | 9 Hotkey-Slots                                |
| `MouseSkillBar.tsx`        | LMB/RMB-Skill-Bindings                        |
| `CharacterView.tsx`        | Paperdoll + Stats + Stat-Punkt-Vergabe (`C`)  |
| `ClassSelectModal.tsx`     | First-Login Klassen-Wahl (Phase 2)            |
| `SkillTreePanel.tsx`       | Skill-Tree, Skill-Punkte, Casten (`K`, Phase 2/3) |
| `ItemTooltip.tsx`          | Affix-Anzeige, Rarity-Farben (`rarity.ts`)    |
| `TradePanel.tsx`           | Markt-Orders, Trading                         |
| `Minimap.tsx`              | Zonen-Overview                                |
| `WaypointTravel.tsx`       | Zonen-Wechsel über freigeschaltete Punkte     |

### 3D-Welt

| Komponente            | Funktion                                  |
| --------------------- | ----------------------------------------- |
| `Player.tsx`          | Eigener Spieler, Input-Handler, Hotkeys   |
| `OtherPlayers.tsx`    | Andere Spieler                            |
| `Enemies.tsx`         | Zombies/Skelette/Fallen ones              |
| `LootDrops.tsx`       | Bodenloot                                 |
| `TradingPostMesh.tsx` | Marktstände                               |
| `Roads.tsx` / `Trees.tsx` / `Water.tsx` / `Terrain.tsx` | Welt |
| `FollowCamera.tsx`    | Kamera-Logik                              |

### Hotkeys

| Taste     | Aktion                |
| --------- | --------------------- |
| `WASD`    | Bewegung              |
| `B`       | Tausch-Panel          |
| `C`       | Character-View        |
| `I`       | Inventar              |
| `K`       | Skill-Tree            |
| `1`–`9`   | Action-Bar            |
| LMB / RMB | Mouse-Skill           |

---

## 9. Bisher umgesetzte Features

### Welt & Wirtschaft (Basis)
- 3 Zonen mit Waypoint-Travel (Town / Wilderness / BurialGrounds)
- Resource-Nodes (gather)
- Spieler-Märkte mit Buy/Sell-Orders, Order-Cancel, Order-Fill
- Commodity-Preise mit Drift / Tick-Updates
- Mission-Board + akzeptierte Missionen
- Reputation-System
- Trade-History (persistent)

### D2-Style RPG
- HP / Mana / Globes
- Stats: Str / Dex / Vit / Eng + freie Stat-Punkte beim Level-Up
- XP-Kurve, Level-Up
- Items mit 5 Rarities, gerollten Affixen
- Equipment (10 Slots, Paperdoll)
- 5 Bags mit Drag-and-Drop, Item-Drop, Equip/Unequip
- ActionBar (9 Slots) + Mouse-Bindings (LMB/RMB)
- Loot-Drops mit Pickup
- Gegner-AI (3 Kinds), Spawning, Death + Despawn
- Combat (Klick-Angriff, mouse_button-Differenzierung)

### Phase 1 — Klassen & Skill-Tree (Backend)
- 3 Klassen (Barbar, Zauberin, Totenbeschwörer) mit Base-Stats und
  Starter-Skill.
- 9-Skill-Catalog, Allocation/Cooldown-System.
- Tauri-Commands `select_class`, `allocate_skill`, `cast_skill`.
- DB-Migration für `class_id`, `allocated_skills`, `unspent_skill_points`,
  `skill_cooldowns`, `active_buffs`.

### Phase 2 — Frontend Wiring
- Types und Catalogs gespiegelt (`src/types`, `src/data/classes.ts`).
- Store mit `classId`, `allocatedSkills`, `unspentSkillPoints`,
  `skillCooldowns`, `activeBuffs`, `skillTreeOpen` + Actions.
- `ClassSelectModal` (First-Login).
- `SkillTreePanel` mit Allocation und Nearest-Enemy-Casting.
- Hotkey `K` in `Player.tsx`, HUD-Hint, Mount in `App.tsx`.

### Phase 3 — Damage Model
- 6 `DamageType`s + 5 `DamageTag`s.
- `DamageInstance` als Schaden-Atom.
- `Resistances` (clamp 75 %, negative = Verwundbarkeit).
- `DotInstance` + `tick_dots`.
- `Enemy` und `PlayerState` haben `resistances` + `dots` + JSONB-Persistenz.
- `EnemyHit` als typed Output von `tick_enemies`.
- Pro `EnemyKind`: eigene `attack_damage_type`, `poison_on_hit`,
  `resistances`-Profile.
- `cast_skill` baut `DamageInstance` aus `damage_type`/`tags` des Skills.
- `apply_dot_to_enemy` mit Replace-Regel (neuer Total ≥ existierender).
- Skill-Catalog komplett mit `damage_type`/`tags` versehen.
- `amplify_damage` als „Giftwolke" aktiviert (Poison-DoT).
- Frontend-Mirror: Types, Catalog, Store, `SkillTreePanel`-Targeting +
  Effekt-Label.

### Phase 4 — GOAP (Goal-Oriented Action Planning)
- Vollständiger A\*-Planner in `ai/goap/` (action.rs · condition.rs ·
  config.rs · goal.rs · resolver.rs · runtime.rs · world_state.rs).
- JSON-authored Agents (`data/goap/agents.json`): Goals + Actions + initiale
  WorldState pro Archetype-Id.
- Sensoren schreiben `WorldKey`s wie `has_target`, `in_attack_range`,
  `hp_percent`. Effekte deklarieren versprochene State-Änderungen.
- `EnemyArchetype.ai` schaltet pro Archetype zwischen `simple_chase` und
  `goap`. Fallback auf `simple_chase`, wenn keine Agent-Config gefunden wird.
- Details: [AI.md](AI.md).

### Phase 5 — Boids-Steering
- `ai/boids.rs` — Reynolds-Flocking (Cohesion · Separation · Alignment) als
  Movement-Layer unter dem GOAP-`move_to_target`-Behaviour.
- `BoidSample`-Snapshot pro Tick (Pre-Mutation), pro Enemy einmalig
  konsultiert — keine `&mut`-Konflikte.
- Cross-Zone-Filter: Nachbarn aus anderer Zone werden ignoriert (per
  `ZoneId`-Gleichheit).

### Phase 6 — Datengetriebene Zonen + D2 Act 1
- `ZoneId` von 3-Variant-Enum auf `Arc<str>`-Newtype migriert (kein Copy mehr,
  cheap Clone via Arc-RefCount).
- `Zone`-Struct erweitert um `act: u8` und `neighbors: Vec<ZoneId>`.
- `data/zones.json` enthält 31 Zonen — Rogue Encampment, Blood Moor, Cold
  Plains, Stony Field, Tristram, Underground Passage, Dark Wood, Black Marsh,
  Forgotten Tower, Tamoe Highland, Pit, Monastery Gate, Outer Cloister,
  Barracks, Jail L1–L3, Inner Cloister, Cathedral, Catacombs L1–L4 …
- 9 D2-canonische Wegpunkte (Rogue Encampment, Cold Plains, Stony Field, Dark
  Wood, Black Marsh, Outer Cloister, Jail L1, Inner Cloister, Catacombs L2).
- Boot-Validierung: panic auf dupes / dangling neighbours / fehlende Town.
- Persistenz: Spalte bleibt `TEXT`. `ZoneId::from_legacy` migriert Phase-1
  Saves transparent.
- Details: [ZONES.md](ZONES.md).

---

## 10. Verify-Workflow

```bash
# Rust-Workspace (game + protocol + server)
cargo check --workspace --message-format short

# Tauri-Crate (lädt das Frontend)
cd src-tauri && cargo check --message-format short

# Frontend-Typecheck
npx tsc --noEmit

# Dev-Build (Hot-Reload Frontend + Rust)
npm run tauri:dev

# Production-Build
npm run tauri:build
```

---

## 11. Offene Punkte / Roadmap

### Direkt anschließend an Phase 3
- **Summoning** (`raise_skeleton`): Pet-Entity als `Enemy`-Variante mit
  `owner_player_id`, AI greift fremde Gegner an.
- **Trap-Entities**: statische Welt-Objekte mit AoE-Tick-Schaden.
- **Lightning-Skill**: Type existiert, noch kein Skill rollt ihn aus.
- **Curses**: Necromancer-Schule mit `damage_taken_amp` als Player-Buff /
  Enemy-Debuff.

### UI
- HUD: aktive `dots` und `resistances` anzeigen (z. B. in Character-View).
- Skill-Tree: Damage-Type-Icons + Resistenz-Tags pro Skill rendern.

### Multiplayer / Performance
- Delta-Compression auf Felder, die sich tatsächlich änderten.
- AOI-Filtering: weit entfernte Spieler/Gegner nicht broadcasten.

### Content
- Mehr Enemy-Kinds mit unterschiedlichen Resistenz-Profilen.
- Endgame-Zone mit Boss-Mechaniken (mehrere Damage-Types pro Phase).

---

## 12. Zusammenfassung

Aktueller Stand: **vollwertige server-authoritative Wirtschafts- und
RPG-Simulation** mit 3D-Frontend, persistenter Datenbank, typed Damage Pipeline
mit Resistenzen und DoTs, vollständigem Klassen- und Skill-System, sowie sauber
gespiegeltem Read-Only-Frontend. Alle drei Compile-Gates sind grün.
