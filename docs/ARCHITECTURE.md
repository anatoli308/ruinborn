# TradeWars — Architektur

## Übersicht

TradeWars ist ein **server-autoritatives MMO**. Jegliche Spiellogik läuft ausschließlich auf dem Rust-Server. Der Client (React + Three.js) ist ein reiner Renderer ohne eigene Spiellogik.

## Schichten

```
┌──────────────────────────────────────────┐
│  React / R3F Components (Rendering)      │  ← Reine Anzeige
├──────────────────────────────────────────┤
│  Zustand Store (Read-Only Mirror)        │  ← Server-State gespiegelt
├──────────────────────────────────────────┤
│  WebSocket Transport (wsTransport.ts)    │  ← JSON Messages, Reconnect
├──────────────────────────────────────────┤
│  tradewars-protocol (Rust Crate)         │  ← ClientMessage / ServerMessage
├──────────────────────────────────────────┤
│  tradewars-server (WebSocket Server)     │  ← Tokio, Connection Handler
├──────────────────────────────────────────┤
│  tradewars-game (Game Logic Library)     │  ← Single Source of Truth
└──────────────────────────────────────────┘
```

## Crate-Abhängigkeiten

```
tradewars-game          ← Keine Abhängigkeiten auf I/O oder Netzwerk
    ↑
tradewars-protocol      ← Importiert nur Typen aus tradewars-game
    ↑
tradewars-server        ← Nutzt game + protocol, fügt Tokio/WebSocket hinzu
```

**Regel:** Abhängigkeiten zeigen immer nach innen. Die Game Logic kennt weder Server noch Client.

## Crate-Verantwortlichkeiten

### `tradewars-game`
- Definiert alle Datenstrukturen: `GameState`, `PlayerState`, `Commodity`, `TradingPost`, `MarketEvent`
- Enthält die gesamte Spielsimulation: `advance_tick()`, `move_player()`, `execute_trade()`
- Erzeugt Snapshots: `build_player_snapshot()`, `build_delta_snapshot()`
- **Kein I/O**, kein Netzwerk, kein Serialisierungs-Framework-Lock-in

### `tradewars-protocol`
- Definiert `ClientMessage` (Frontend → Server) und `ServerMessage` (Server → Frontend)
- JSON-basiert mit `serde` Tag-Enums (`"cmd"` / `"type"`)
- Shared Contract zwischen Server und Client

### `tradewars-server`
- Tokio-basierter async WebSocket Server
- TCP Listener auf `0.0.0.0:9000`
- Pro Connection: eigener Task mit `mpsc` Channel für Outbound-Messages
- Tick-Loop als Background-Task
- Verwaltet `Arc<RwLock<GameState>>` als geteilten State

### `src-tauri` (Optional)
- Minimale Desktop-Shell — nur `tauri::Builder` ohne IPC-Commands
- Der Client verbindet sich per WebSocket, nicht per Tauri IPC
- Kann weggelassen werden; der Client läuft auch im Browser

## Datenfluss

```
Spieler drückt WASD
    ↓
Player.tsx: Client-side Prediction (sofortige Anzeige)
    ↓ sendMove(dx, dz)
gameStore.ts → wsTransport.send({ cmd: "move", dx, dz })
    ↓ WebSocket
tradewars-server: handle_connection()
    ↓
game::move_player(&mut game, pid, dx, dz)
    ↓
GameState mutiert (Single Source of Truth)
    ↓ Nächster 50ms Broadcast-Tick
game::build_delta_snapshot() → ServerMessage::Delta
    ↓ WebSocket
wsTransport.onMessage() → gameStore mapDelta()
    ↓
Zustand Store aktualisiert
    ↓
React re-rendert betroffene Components
    ↓
Player.tsx: Reconcile predicted → server position
```

## Zustand Store als Read-Only Mirror

Der Zustand Store hält **keine eigene Spiellogik**. Er:
1. Empfängt Server-Messages (`"state"`, `"delta"`, `"trade_result"`, `"welcome"`)
2. Mapped `snake_case` → `camelCase`
3. Stellt `send*` Actions bereit, die WebSocket-Messages absenden
4. Wird von React Components per Selector gelesen

## Neues Feature hinzufügen

Immer in dieser Reihenfolge:

1. **Game Logic** → `tradewars-game`: State + Mutations-Funktion
2. **Protocol** → `tradewars-protocol`: Neuer `ClientMessage`/`ServerMessage` Variant
3. **Server** → `tradewars-server`: Command-Handler + Emit
4. **Store** → `gameStore.ts`: Mapping + `send*` Action
5. **Types** → `types/index.ts`: Frontend-Interface
6. **UI** → React Component: Store lesen, Action senden, rendern
