# TradeWars – Wirtschaftssimulation

Eine 3D-Marktsimulation als Desktop-App. Starte als Händler, reise zwischen Handelsposten und baue dein Trading-Imperium auf.

## Voraussetzungen

- **Node.js** ≥ 18 (empfohlen: 22+)
- **Rust** ≥ 1.70 (für Tauri-Backend)
- **npm** ≥ 9

## Installation

```bash
npm install
```

## Development

### Nur Frontend (Vite Dev Server)

```bash
npm run dev
```

Öffnet auf [http://localhost:1420](http://localhost:1420)

### Vollständig mit Tauri (Desktop-App)

```bash
npm run tauri:dev
```

Startet die Desktop-App mit Hot-Reload.

## Production Build

```bash
npm run tauri:build
```

Das fertige Installationspaket liegt unter `src-tauri/target/release/bundle/`.

## Steuerung

| Taste | Aktion |
|-------|--------|
| **W A S D** | Spieler bewegen |
| **E** | Handelsposten öffnen (wenn in der Nähe) |
| **Leertaste** | Pause / Fortsetzen |
| **1 / 2 / 3** | Geschwindigkeit (×1 / ×2 / ×5) |
| **Esc** | Handelspanel schließen |

## Techstack

- **Frontend:** React 19, TypeScript, Three.js (via @react-three/fiber + @react-three/drei)
- **State:** Zustand
- **Desktop:** Tauri 2 (Rust)
- **Build:** Vite 6
- **Styling:** CSS (custom game UI)

## Projektstruktur

```
src/
├── components/
│   ├── world/          # 3D-Szene: Terrain, Bäume, Wasser, Straßen, Handelsposten, Spieler
│   ├── ui/             # HUD, Handelspanel, Inventar, Minimap
│   ├── GameWorld.tsx    # R3F Canvas + Szene
│   └── GameTicker.tsx   # Markt-Simulation pro Tick
├── store/
│   └── gameStore.ts     # Zustand-Store (Spielzustand + Marktlogik)
├── types/
│   └── index.ts         # TypeScript-Interfaces
└── styles/
    └── index.css        # Game UI Styles

src-tauri/
├── src/
│   ├── market.rs        # Rust Markt-Engine
│   ├── lib.rs           # Tauri Commands
│   └── main.rs          # Windows Entry Point
└── tauri.conf.json      # Tauri Konfiguration
```
