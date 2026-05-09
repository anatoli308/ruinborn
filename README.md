# Ruinborn — Server-Authoritative Isometric Action MMO RPG

> *Born from ruin.* Ein D2-LoD-inspiriertes ARPG mit persistenter Welt, isometrischer 3D-Sicht und 100 % server-autoritativer Simulation.

Spielloop: **Monster töten → Loot finden → Build verbessern → Wegpunkte freischalten → tiefere Dungeons**. Wirtschaft, Trading-Posts und Markt-Orders sind als Layer über dem Combat-Core eingebaut, nicht als Hauptfokus.

## Genre & Pillars

- **Action-Combat** — 20 Hz Tick, Klick-Targeting, Skill-Casting, LMB/RMB-Bindings
- **Loot & Buildcraft** — 5 Rarities, gerollte Affixe, Paperdoll, Skill-Tree, Stat-Allocation
- **Persistente MMO-Welt** — Postgres-Persistenz, alle Spieler im selben `GameState`
- **D2 Act 1** — 31 datengetriebene Zonen, 9 Wegpunkte, Town/Wilderness/Dungeon-Hierarchie
- **Server-Autorität** — der Client ist ein reiner Renderer; jede Mutation läuft im Rust-Server

## Quickstart

```bash
# Voraussetzungen: Node.js ≥ 18, Rust ≥ 1.70, Postgres erreichbar
npm install

# Terminal 1 — dedizierter Game-Server (WS auf :9000)
npm run server:dev

# Terminal 2a — Browser-Client
npm run dev                 # http://localhost:1420

# oder 2b — Tauri Desktop-Shell
npm run tauri:dev
```

Production:

```bash
npm run server:build
npm run tauri:build
```

## Steuerung

| Taste     | Aktion                                  |
| --------- | --------------------------------------- |
| `WASD`    | Bewegung                                |
| `LMB/RMB` | Gebundener Skill bzw. Basis-Angriff     |
| `1`–`9`   | Action-Bar Hotkeys                      |
| `I`       | Inventar                                |
| `C`       | Character / Stats                       |
| `K`       | Skill-Tree                              |
| `B`       | Trade-Panel (an Trading-Post)           |
| `M`       | Wegpunkt-Travel                         |
| `Esc`     | aktuelles Panel schließen               |

## Tech-Stack

| Schicht           | Technologie                                                          |
| ----------------- | -------------------------------------------------------------------- |
| **Game Server**   | Rust · Tokio · `tokio-tungstenite` (WebSocket) · SeaORM · Postgres   |
| **Game Logic**    | `ruinborn-game` Crate (pure, kein I/O)                              |
| **Protokoll**     | `ruinborn-protocol` Crate (JSON `ClientMessage` / `ServerMessage`)  |
| **Frontend**      | React 19 · TypeScript · Vite 6 · Three.js (R3F + Drei) · Zustand    |
| **Desktop-Shell** | Tauri 2 (optional; reiner WS-Client, kein IPC)                       |
| **AI**            | GOAP (A\* Planner, JSON-authored Agents) + Reynolds-Boids Steering   |
| **Daten**         | JSON-Files für Zonen, Enemies, GOAP-Agents (in `data/`)              |

## Workspace

```
ruinborn/
├── crates/
│   ├── ruinborn-game/       # Pure Sim-Library
│   │   ├── data/
│   │   │   ├── zones.json        # Phase 6: Full D2 Act 1 (31 Zonen)
│   │   │   ├── enemies.json      # Archetype-Registry
│   │   │   └── goap/agents.json  # GOAP-Configs pro Archetype
│   │   └── src/
│   │       ├── world.rs       # ZoneId, ZoneKind, Catalogue, Wegpunkte
│   │       ├── combat.rs      # Enemy, Tick, Aggro, Damage-Pipeline
│   │       ├── damage.rs      # DamageType/Tag, Resistances, DotInstance
│   │       ├── skills.rs      # SkillDef-Catalog, cast_skill
│   │       ├── classes.rs     # Barbarian / Sorceress / Necromancer
│   │       ├── progression.rs # XP-Curve, Level-Up
│   │       ├── items.rs       # Rarity, Affixes, Bags, Equipment
│   │       ├── enemy_archetype.rs
│   │       ├── ai/
│   │       │   ├── boids.rs   # Reynolds-Flocking
│   │       │   └── goap/      # Planner + Runtime
│   │       └── market.rs      # Wirtschaft, GameState, advance_tick
│   ├── ruinborn-protocol/   # Wire-Format
│   └── ruinborn-server/     # Tokio + WS + SeaORM-Persistenz
├── src/                      # Frontend
├── src-tauri/                # Desktop-Shell
└── docs/                     # Architektur & Reference-Docs
```

## Dokumentation

| Doc                                                          | Inhalt                                                       |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| [docs/OVERVIEW.md](docs/OVERVIEW.md)                         | Gesamt-Setup, Phasen 1–6, Persistenz, UI                     |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)                 | Schichten, Crate-Abhängigkeiten, Datenfluss                  |
| [docs/NETWORKING.md](docs/NETWORKING.md)                     | Tick-Raten, Delta-Snapshots, Client-Side-Prediction          |
| [docs/DAMAGE_MODEL.md](docs/DAMAGE_MODEL.md)                 | Phase 1–3: Klassen, Skills, Damage-Pipeline                  |
| [docs/AI.md](docs/AI.md)                                     | GOAP + Boids — JSON-Agents, Planner, Steering                |
| [docs/ZONES.md](docs/ZONES.md)                               | Datengetriebene Zonen, D2 Act 1, Wegpunkt-Graph              |
| [docs/REFERENCE_VS_SOURCE.md](docs/REFERENCE_VS_SOURCE.md)   | Vergleich gegen die C++-Referenz-Sources                     |
| [docs/IDEEN.md](docs/IDEEN.md)                               | Roadmap & geplante Features                                  |

## Status

- ✅ Phasen 1–6 abgeschlossen (Klassen → Skills → Damage → GOAP → Boids → Data-Driven Zones)
- ✅ `cargo check --workspace` clean · 47/47 Tests grün · Server kompiliert sauber
- ⏳ Roadmap-Schwerpunkte: Summons, Curses, Boss-Encounters, Auction-House, Multiplayer-AOI

## Lizenz

Privates Projekt — kein Public License.
