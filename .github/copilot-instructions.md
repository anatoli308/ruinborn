# Copilot Instructions for TradeWars

## Background Information
This file contains the coding standards, architectural principles, and design patterns for the TradeWars project — a server-authoritative Wirtschaftssimulation built with **Tauri 2 (Rust backend)** and **React + Three.js/R3F (frontend)**. It serves as a guideline for all developers to ensure consistency, maintainability, and clarity. The instructions cover decision-making principles, coding rules, clean coding standards, project architecture, server-authoritative patterns, and conventions. Always use the latest stable versions of Rust, TypeScript, React, and Three.js features where appropriate.

## Game Vision
TradeWars is a tick-based economy simulation where players trade commodities, manage firms, influence markets through supply/demand, execute financial strategies (stocks, futures, policies), and compete on leaderboards. The game features a 3D world rendered with React Three Fiber, with all game logic running server-side in Rust. Future scope includes multiplayer, AI agents, financial markets, and emergent macro-economics (see `docs/IDEEN.md` for the full feature roadmap).

## Decision-Making Principles
- For background tasks or long decision tasks use Python and not PowerShell. PowerShell is only for short scripts and quick fixes, not for complex logic or data processing.
- Always prefer clear, maintainable code over clever one-liners. Readability is more important than brevity.
- When in doubt whether logic belongs in Rust or TypeScript, it belongs in **Rust**. The frontend is a thin client.
- Prefer Rust's type system and `Result<T, E>` for error handling over string-based errors.

## Important Developer Coding Rules

### Rust (Backend / Server)
- Always use explicit types — avoid `_` type inference where the type is not immediately obvious.
- Use `///` doc comments for all public structs, enums, functions, and methods.
- All game state mutations happen exclusively in Rust. The frontend NEVER computes or mutates game state.
- Use `#[tauri::command]` for all IPC endpoints. Every command that mutates state must emit a `"game-state"` event after mutation.
- Prefer `Mutex<GameState>` for thread-safe state access. Always `drop()` the lock before calling `app.emit()`.
- No `unwrap()` in command handlers — use `map_err(|e| e.to_string())?` for error propagation.
- Use constants for magic numbers (e.g., `WORLD_BOUND`, `POST_INTERACTION_RANGE`, `PRICE_HISTORY_MAX`).

### TypeScript / React (Frontend / Client)
- Use explicit TypeScript types — avoid `any`. Define interfaces for all data structures.
- The Zustand store is a **read-only mirror** of the Rust GameState. No simulation logic in the store.
- All player actions go through Tauri `invoke()` commands. Never mutate game-relevant state locally.
- Use `listen()` from `@tauri-apps/api/event` for realtime state updates from Rust.
- Map Rust `snake_case` field names to TypeScript `camelCase` in the store mapping layer.
- React components are pure renderers: they read from the store and send commands to Rust.
- No polling, no timers, no `setInterval` for game logic. The Rust tick loop drives everything.

### General
- No `console.log` in production code — use it only for debugging, remove before commit.
- No hardcoded strings for commodity IDs, trading post IDs, or event names — use constants or enums.
- No Polling, No Flag Checks: Vermeide Polling-Logik und Flag-based Ablaufsteuerung; nutze stattdessen State Machines, Event-Driven Logic oder Callbacks.

## Clean Coding Standards
- **KISS (Keep It Simple, Stupid)**: Bevorzuge einfache, klare Lösungen gegenüber komplexen; vermeide Over-Engineering; jede Funktion/Struct sollte eine klare, verständliche Aufgabe haben.
- **DRY (Don't Repeat Yourself)**: Keine Code-Duplikation; extrahiere wiederholte Logik in gemeinsame Funktionen/Module; nutze Traits und Generics sinnvoll.
- **YAGNI (You Aren't Gonna Need It)**: Implementiere nur Features, die aktuell benötigt werden; keine spekulativen Erweiterungen; halte Code fokussiert auf aktuelle Requirements. Siehe `docs/IDEEN.md` für geplante Features — implementiere sie erst wenn explizit verlangt.
- **Single Responsibility Principle (SRP)**: Jede Datei/Modul hat genau eine Verantwortung. Kein Mixed Concerns — ein Modul simuliert ODER rendert ODER mapped State, nie beides.
- **Separation of Concerns**: Klare Trennung zwischen Simulation (Rust), IPC (Tauri Commands/Events), State-Mapping (Zustand Store), und Rendering (React/R3F Components). Diese Schichten dürfen sich nie vermischen.
- **Clean Architecture**: Abhängigkeiten zeigen immer nach innen. Game Logic hat keine IPC-Dependencies. IPC-Layer hat keine Rendering-Dependencies. Store hat keine Simulation.
- **Explicit over Implicit**: Keine magischen Strings/Numbers; explizite Typen; klare Funktionsnamen; Konstanten für wiederholte Werte.
- **Fail Fast**: Validierung früh durchführen — in Rust-Commands am Eingang, Guard Clauses am Anfang von Funktionen; klare Error-Messages (deutsch für Spieler-sichtbare, englisch für Dev-Logs).
- **Composition over Inheritance**: React Components nutzen Composition; Rust nutzt Traits statt Vererbung.
- **Immutability where possible**: Rust `&self` bevorzugen wo möglich; React Components sind reine Funktionen; Zustand Store-Updates sind immutable via `set()`.

## Project Architecture

### Tech Stack
- **Backend**: Rust + Tauri 2 (Desktop App Framework, IPC via Commands/Events)
- **Frontend**: React 19 + TypeScript + Vite
- **3D Rendering**: Three.js via React Three Fiber (R3F) + Drei
- **State Management**: Zustand (thin client, mirrors Rust state)
- **Styling**: Tailwind CSS + PostCSS

### Server-Authoritative Architecture
```
Spieler-Input (Tastatur/Maus)
    ↓
React Component
    ↓ invoke("command_name", { args })
Tauri IPC
    ↓
Rust Command Handler
    ↓ game_logic::function(&mut game, args)
Game Logic Module  ←── Single Source of Truth
    ↓
app.emit("game-state", &snapshot)
    ↓
Tauri Event System
    ↓ listen("game-state")
Zustand Store  ←── Read-Only Mirror
    ↓ mapServerState() [snake_case → camelCase]
React Components  ←── Pure Renderers
```

## Tick System & Realtime
- Die Tick-Simulation läuft in einem **Rust Background Thread** (nicht im Frontend).
- Tick-Intervall: `1 Sekunde / speed`.
- Der Thread checkt hochfrequent ob ein Tick fällig ist.
- Nach jedem Tick wird der komplette `GameState` als Event emittiert.
- Input-Commands (z.B. Bewegung) emittieren sofort für responsive Steuerung.

## Patterns & Conventions

### Single Source of Truth
- **Rust `GameState`** ist die einzige Wahrheit. Kein State lebt nur im Frontend.
- Events (`"game-state"`) sind vollständige Snapshots, keine Deltas.
- Der Zustand Store ist ein Read-Only Mirror — keine lokale Mutation, keine optimistischen Updates.
- UI-Components fragen Daten beim Store ab, nie direkt beim Server.

### Klare Trennungen (Separation of Concerns)
- **Game Logic (Rust)**: Hält State, simuliert, validiert. Single Source of Truth. Kein IPC-Code.
- **IPC Layer (Rust)**: Commands, Event-Emission, App-Setup. Keine Geschäftslogik.
- **Store (TypeScript)**: State-Mapping + IPC-Wrapper (`send*` Actions). Keine Logik, keine Berechnung.
- **UI Components (React)**: Reines Rendering. Lesen Store, senden Commands. Keine State-Berechnung.
- **3D World Components (R3F)**: Nur visuell. Positionen und Daten kommen ausschließlich vom Store.

### State Machines
- Für komplexe, zustandsbehaftete Abläufe (z.B. Verbindungsprozesse, Ladesequenzen, mehrstufige Aktionen) State Machines verwenden statt Flag-Checks oder verschachtelter if/else-Logik.
- In Rust: Enums mit zugehörigen Daten als States; Übergänge sind explizite Funktionen.
- In TypeScript: Zustand-basierte State-Felder mit klar definierten Übergangsfunktionen.
- States steuern nur Transitionen — Business-Logik bleibt im zugehörigen Manager/Handler.

### Service Decomposition
- Komplexe Module können in interne Sub-Services aufgeteilt werden für bessere Separation of Concerns.
- Aufbau: **Manager** orchestriert → **interne Services** erledigen spezifische Operationen (laden, validieren, berechnen).
- Interne Services halten keinen persistenten State — sie bekommen Daten als Parameter und geben Ergebnisse zurück.
- Beispiel: Ein `FirmManager` orchestriert → `ProductionCalculator`, `InventoryValidator`, `PriceApplier` als reine Funktionen/Module.

### Dependency Management
- Game Logic Module haben keine Abhängigkeit auf IPC oder Frontend-Code.
- IPC-Commands haben keine Abhängigkeit auf Rendering oder Store-Logik.
- Komplexe Systeme erhalten ihre Abhängigkeiten via Parameter (Dependency Injection), nicht via globale Singletons.
- In Rust: State wird via `&mut GameState` durchgereicht — kein globaler Zugriff.

### Event-Driven Logic
- Keine Polling-Schleifen im Frontend für Game-State. Der Rust Tick-Loop pusht State.
- Keine Timer oder `setInterval` für Spiellogik — der Rust-Thread ist der einzige Takt.
- Reaktionen auf State-Änderungen laufen über den Zustand Store (reaktiv via Subscriptions).
- Rust-seitig: Background-Thread emittiert Events; Commands reagieren auf Inputs sofort.

### IPC Patterns
- **Commands** (Frontend → Rust): `invoke("command_name", { args })` — für Player-Inputs und Aktionen.
- **Events** (Rust → Frontend): `emit("game-state", &snapshot)` — für realtime State-Pushes.
- Command-Argumente: TypeScript `camelCase` → Tauri auto-converts to Rust `snake_case`.
- Alle Commands geben `Result<T, String>` zurück. Fehler werden user-freundlich (deutsch) formatiert.

### Naming Conventions
- **Rust**: `snake_case` für Funktionen/Variablen, `PascalCase` für Structs/Enums, `SCREAMING_SNAKE_CASE` für Konstanten.
- **TypeScript**: `camelCase` für Variablen/Funktionen, `PascalCase` für Types/Interfaces/Components, `SCREAMING_SNAKE_CASE` für Konstanten.
- **IPC Commands**: `snake_case` in Rust (`move_player`), camelCase in TypeScript args (`{ commodityId }`).
- **Tauri Events**: kebab-case (`"game-state"`).
- **Files**: `PascalCase.tsx` für Components, `camelCase.ts` für Stores/Utils, `snake_case.rs` für Rust modules.

### Error Handling
- Rust Commands: `Result<T, String>` — konvertiere alle Errors via `.map_err(|e| e.to_string())`.
- Spieler-sichtbare Fehler: Deutsch, freundlich (z.B. `"❌ Nicht genug Gold!"`).
- Dev-Logs: Englisch.
- Frontend: `.catch(console.error)` für fire-and-forget Commands, `try/catch` für Commands mit UI-Feedback.

### Adding New Features
1. **Game Logic** → Rust: Neuen State zum `GameState` hinzufügen, Mutations-Funktionen schreiben.
2. **IPC Command** → Rust: `#[tauri::command]` handler, in `invoke_handler!` registrieren, `emit("game-state")` nach Mutation.
3. **Store Mapping** → TypeScript: `ServerGameState` Interface erweitern, `mapServerState()` updaten, `send*` Action hinzufügen.
4. **TypeScript Types**: Interface für Frontend-Representation.
5. **UI Component** → React: Store lesen, `send*` aufrufen, rendern.
6. **Immer in dieser Reihenfolge**: Rust → Command → Store → Types → Component.

## Build & Development
- **Dev**: `npm run tauri:dev` — startet Vite Dev Server + Tauri App mit Hot Reload.
- **Build**: `npm run tauri:build` — Production Build.
- **Rust Check**: `cd src-tauri && cargo check` — schneller Compile-Check ohne Build.
- **TypeScript Check**: `npx tsc --noEmit` — Type-Check ohne Build.
- **Dependencies**: Frontend in `package.json`, Backend in `src-tauri/Cargo.toml`.
