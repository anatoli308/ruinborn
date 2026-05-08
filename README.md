# TradeWars – Social Sandbox MMO

Ein tick-basiertes Wirtschafts-MMO, in dem Spieler Waren handeln, Märkte beeinflussen, Firmen managen und gegeneinander auf Leaderboards antreten. Der Server ist die einzige Autorität — der Client ist ein reiner Renderer.

## Voraussetzungen

- **Node.js** ≥ 18 (empfohlen: 22+)
- **Rust** ≥ 1.70 (für Server + optionaler Tauri-Desktop-Shell)
- **npm** ≥ 9

## Installation

```bash
npm install
```

## Development

### 1. Server starten (Pflicht)

```bash
npm run server:dev
```

Startet den dedizierten WebSocket-Gameserver auf `ws://localhost:9000`.

### 2a. Frontend im Browser

```bash
npm run dev
```

Öffnet auf [http://localhost:1420](http://localhost:1420) — verbindet sich automatisch zum Server.

### 2b. Oder: Desktop-App mit Tauri

```bash
npm run tauri:dev
```

Tauri-Shell mit Hot-Reload (verbindet sich ebenfalls per WebSocket zum Server).

## Production Build

```bash
npm run server:build          # Server-Binary
npm run tauri:build            # Desktop-Installer (optional)
```

## Steuerung

| Taste | Aktion |
|-------|--------|
| **W A S D** | Spieler bewegen |
| **E** | Handelsposten öffnen/schließen |
| **Esc** | Handelspanel schließen |

## Techstack

| Schicht | Technologie |
|---------|-------------|
| **Game Server** | Rust, Tokio, WebSocket (`tokio-tungstenite`) |
| **Game Logic** | `tradewars-game` Crate (reine Simulation, kein I/O) |
| **Protokoll** | `tradewars-protocol` Crate (JSON, `ClientMessage`/`ServerMessage`) |
| **Frontend** | React 19, TypeScript, Three.js (R3F + Drei) |
| **State** | Zustand (read-only Mirror des Servers) |
| **Desktop** | Tauri 2 (optionale Shell, keine IPC — reiner WebSocket-Client) |
| **Build** | Vite 6, Cargo |
| **Styling** | Tailwind CSS |

## Projektstruktur

```
crates/
├── tradewars-game/        # Game Logic Library (Simulation, keine I/O)
│   └── src/market.rs      # Commodities, TradingPosts, Economy Tick, Trades
├── tradewars-protocol/    # Shared Message Types (Client ↔ Server)
│   └── src/lib.rs         # ClientMessage, ServerMessage, DeltaSnapshot
└── tradewars-server/      # Dedizierter WebSocket Game Server
    └── src/main.rs        # Tokio async, Tick Loop, Connection Handler

src/                       # Frontend (React + R3F)
├── components/
│   ├── world/             # 3D-Szene: Terrain, Bäume, Wasser, Spieler
│   ├── ui/                # HUD, Handelspanel, Inventar, Minimap
│   ├── GameWorld.tsx       # R3F Canvas + Szene
│   └── GameTicker.tsx      # WebSocket-Verbindung initialisieren
├── services/
│   └── wsTransport.ts     # WebSocket Transport Layer (Connect, Send, Reconnect)
├── store/
│   └── gameStore.ts       # Zustand Store (Server-Mirror + Actions)
├── types/
│   └── index.ts           # Frontend TypeScript Interfaces
└── styles/
    └── index.css          # Tailwind + Game UI Styles

src-tauri/                 # Optionale Tauri Desktop Shell
└── src/lib.rs             # Minimaler Wrapper (kein IPC, kein Game-State)
```

## Architektur

Siehe [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) für die vollständige Architektur-Dokumentation.

## Networking & Performance

Siehe [docs/NETWORKING.md](docs/NETWORKING.md) für Details zu Tick-Rates, Delta-Updates und Client-side Prediction.

## Feature-Roadmap

Siehe [docs/IDEEN.md](docs/IDEEN.md) für geplante Features (Firmen, Finanzmärkte, AI-Agenten, Multiplayer-Strategien).
