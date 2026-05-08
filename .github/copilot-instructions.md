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


---
paths:
  - "**/*.ts"
  - "**/*.tsx"
  - "**/*.js"
  - "**/*.jsx"
---
# TypeScript/JavaScript Coding Style

> This file extends [common/coding-style.md](common/coding-style.md) with TypeScript/JavaScript specific content.

## Types and Interfaces

Use types to make public APIs, shared models, and component props explicit, readable, and reusable.

### Public APIs

- Add parameter and return types to exported functions, shared utilities, and public class methods
- Let TypeScript infer obvious local variable types
- Extract repeated inline object shapes into named types or interfaces

```typescript
// WRONG: Exported function without explicit types
export function formatUser(user) {
  return `${user.firstName} ${user.lastName}`
}

// CORRECT: Explicit types on public APIs
interface User {
  firstName: string
  lastName: string
}

export function formatUser(user: User): string {
  return `${user.firstName} ${user.lastName}`
}
```

### Interfaces vs. Type Aliases

- Use `interface` for object shapes that may be extended or implemented
- Use `type` for unions, intersections, tuples, mapped types, and utility types
- Prefer string literal unions over `enum` unless an `enum` is required for interoperability

```typescript
interface User {
  id: string
  email: string
}

type UserRole = 'admin' | 'member'
type UserWithRole = User & {
  role: UserRole
}
```

### Avoid `any`

- Avoid `any` in application code
- Use `unknown` for external or untrusted input, then narrow it safely
- Use generics when a value's type depends on the caller

```typescript
// WRONG: any removes type safety
function getErrorMessage(error: any) {
  return error.message
}

// CORRECT: unknown forces safe narrowing
function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }

  return 'Unexpected error'
}
```

### React Props

- Define component props with a named `interface` or `type`
- Type callback props explicitly
- Do not use `React.FC` unless there is a specific reason to do so

```typescript
interface User {
  id: string
  email: string
}

interface UserCardProps {
  user: User
  onSelect: (id: string) => void
}

function UserCard({ user, onSelect }: UserCardProps) {
  return <button onClick={() => onSelect(user.id)}>{user.email}</button>
}
```

### JavaScript Files

- In `.js` and `.jsx` files, use JSDoc when types improve clarity and a TypeScript migration is not practical
- Keep JSDoc aligned with runtime behavior

```javascript
/**
 * @param {{ firstName: string, lastName: string }} user
 * @returns {string}
 */
export function formatUser(user) {
  return `${user.firstName} ${user.lastName}`
}
```

## Immutability

Use spread operator for immutable updates:

```typescript
interface User {
  id: string
  name: string
}

// WRONG: Mutation
function updateUser(user: User, name: string): User {
  user.name = name // MUTATION!
  return user
}

// CORRECT: Immutability
function updateUser(user: Readonly<User>, name: string): User {
  return {
    ...user,
    name
  }
}
```

## Error Handling

Use async/await with try-catch and narrow unknown errors safely:

```typescript
interface User {
  id: string
  email: string
}

declare function riskyOperation(userId: string): Promise<User>

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }

  return 'Unexpected error'
}

const logger = {
  error: (message: string, error: unknown) => {
    // Replace with your production logger (for example, pino or winston).
  }
}

async function loadUser(userId: string): Promise<User> {
  try {
    const result = await riskyOperation(userId)
    return result
  } catch (error: unknown) {
    logger.error('Operation failed', error)
    throw new Error(getErrorMessage(error))
  }
}
```

## Input Validation

Use Zod for schema-based validation and infer types from the schema:

```typescript
import { z } from 'zod'

const userSchema = z.object({
  email: z.string().email(),
  age: z.number().int().min(0).max(150)
})

type UserInput = z.infer<typeof userSchema>

const validated: UserInput = userSchema.parse(input)
```

## Console.log

- No `console.log` statements in production code
- Use proper logging libraries instead
- See hooks for automatic detection


---
paths:
  - "**/*.ts"
  - "**/*.tsx"
  - "**/*.js"
  - "**/*.jsx"
---
# TypeScript/JavaScript Hooks

> This file extends [common/hooks.md](common/hooks.md) with TypeScript/JavaScript specific content.

## PostToolUse Hooks

Configure in `~/.claude/settings.json`:

- **Prettier**: Auto-format JS/TS files after edit
- **TypeScript check**: Run `tsc` after editing `.ts`/`.tsx` files
- **console.log warning**: Warn about `console.log` in edited files

## Stop Hooks

- **console.log audit**: Check all modified files for `console.log` before session ends


---
paths:
  - "**/*.ts"
  - "**/*.tsx"
  - "**/*.js"
  - "**/*.jsx"
---
# TypeScript/JavaScript Patterns

> This file extends [common/patterns.md](common/patterns.md) with TypeScript/JavaScript specific content.

## API Response Format

```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
  meta?: {
    total: number
    page: number
    limit: number
  }
}
```

## Custom Hooks Pattern

```typescript
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    const handler = setTimeout(() => setDebouncedValue(value), delay)
    return () => clearTimeout(handler)
  }, [value, delay])

  return debouncedValue
}
```

## Repository Pattern

```typescript
interface Repository<T> {
  findAll(filters?: Filters): Promise<T[]>
  findById(id: string): Promise<T | null>
  create(data: CreateDto): Promise<T>
  update(id: string, data: UpdateDto): Promise<T>
  delete(id: string): Promise<void>
}
```


---
paths:
  - "**/*.ts"
  - "**/*.tsx"
  - "**/*.js"
  - "**/*.jsx"
---
# TypeScript/JavaScript Security

> This file extends [common/security.md](common/security.md) with TypeScript/JavaScript specific content.

## Secret Management

```typescript
// NEVER: Hardcoded secrets
const apiKey = "sk-proj-xxxxx"

// ALWAYS: Environment variables
const apiKey = process.env.OPENAI_API_KEY

if (!apiKey) {
  throw new Error('OPENAI_API_KEY not configured')
}
```

## Agent Support

- Use **security-reviewer** skill for comprehensive security audits


---
paths:
  - "**/*.rs"
---
# Rust Coding Style

> This file extends [common/coding-style.md](common/coding-style.md) with Rust-specific content.

## Formatting

- **rustfmt** for enforcement — always run `cargo fmt` before committing
- **clippy** for lints — `cargo clippy -- -D warnings` (treat warnings as errors)
- 4-space indent (rustfmt default)
- Max line width: 100 characters (rustfmt default)

## Immutability

Rust variables are immutable by default — embrace this:

- Use `let` by default; only use `let mut` when mutation is required
- Prefer returning new values over mutating in place
- Use `Cow<'_, T>` when a function may or may not need to allocate

```rust
use std::borrow::Cow;

// GOOD — immutable by default, new value returned
fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', "_"))
    } else {
        Cow::Borrowed(input)
    }
}

// BAD — unnecessary mutation
fn normalize_bad(input: &mut String) {
    *input = input.replace(' ', "_");
}
```

## Naming

Follow standard Rust conventions:
- `snake_case` for functions, methods, variables, modules, crates
- `PascalCase` (UpperCamelCase) for types, traits, enums, type parameters
- `SCREAMING_SNAKE_CASE` for constants and statics
- Lifetimes: short lowercase (`'a`, `'de`) — descriptive names for complex cases (`'input`)

## Ownership and Borrowing

- Borrow (`&T`) by default; take ownership only when you need to store or consume
- Never clone to satisfy the borrow checker without understanding the root cause
- Accept `&str` over `String`, `&[T]` over `Vec<T>` in function parameters
- Use `impl Into<String>` for constructors that need to own a `String`

```rust
// GOOD — borrows when ownership isn't needed
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

// GOOD — takes ownership in constructor via Into
fn new(name: impl Into<String>) -> Self {
    Self { name: name.into() }
}

// BAD — takes String when &str suffices
fn word_count_bad(text: String) -> usize {
    text.split_whitespace().count()
}
```

## Error Handling

- Use `Result<T, E>` and `?` for propagation — never `unwrap()` in production code
- **Libraries**: define typed errors with `thiserror`
- **Applications**: use `anyhow` for flexible error context
- Add context with `.with_context(|| format!("failed to ..."))?`
- Reserve `unwrap()` / `expect()` for tests and truly unreachable states

```rust
// GOOD — library error with thiserror
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid config format: {0}")]
    Parse(String),
}

// GOOD — application error with anyhow
use anyhow::Context;

fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {path}"))?;
    toml::from_str(&content)
        .with_context(|| format!("failed to parse {path}"))
}
```

## Iterators Over Loops

Prefer iterator chains for transformations; use loops for complex control flow:

```rust
// GOOD — declarative and composable
let active_emails: Vec<&str> = users.iter()
    .filter(|u| u.is_active)
    .map(|u| u.email.as_str())
    .collect();

// GOOD — loop for complex logic with early returns
for user in &users {
    if let Some(verified) = verify_email(&user.email)? {
        send_welcome(&verified)?;
    }
}
```

## Module Organization

Organize by domain, not by type:

```text
src/
├── main.rs
├── lib.rs
├── auth/           # Domain module
│   ├── mod.rs
│   ├── token.rs
│   └── middleware.rs
├── orders/         # Domain module
│   ├── mod.rs
│   ├── model.rs
│   └── service.rs
└── db/             # Infrastructure
    ├── mod.rs
    └── pool.rs
```

## Visibility

- Default to private; use `pub(crate)` for internal sharing
- Only mark `pub` what is part of the crate's public API
- Re-export public API from `lib.rs`

## References

See skill: `rust-patterns` for comprehensive Rust idioms and patterns.


---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---
# Rust Hooks

> This file extends [common/hooks.md](common/hooks.md) with Rust-specific content.

## PostToolUse Hooks

Configure in `~/.claude/settings.json`:

- **cargo fmt**: Auto-format `.rs` files after edit
- **cargo clippy**: Run lint checks after editing Rust files
- **cargo check**: Verify compilation after changes (faster than `cargo build`)


---
paths:
  - "**/*.rs"
---
# Rust Patterns

> This file extends [common/patterns.md](common/patterns.md) with Rust-specific content.

## Repository Pattern with Traits

Encapsulate data access behind a trait:

```rust
pub trait OrderRepository: Send + Sync {
    fn find_by_id(&self, id: u64) -> Result<Option<Order>, StorageError>;
    fn find_all(&self) -> Result<Vec<Order>, StorageError>;
    fn save(&self, order: &Order) -> Result<Order, StorageError>;
    fn delete(&self, id: u64) -> Result<(), StorageError>;
}
```

Concrete implementations handle storage details (Postgres, SQLite, in-memory for tests).

## Service Layer

Business logic in service structs; inject dependencies via constructor:

```rust
pub struct OrderService {
    repo: Box<dyn OrderRepository>,
    payment: Box<dyn PaymentGateway>,
}

impl OrderService {
    pub fn new(repo: Box<dyn OrderRepository>, payment: Box<dyn PaymentGateway>) -> Self {
        Self { repo, payment }
    }

    pub fn place_order(&self, request: CreateOrderRequest) -> anyhow::Result<OrderSummary> {
        let order = Order::from(request);
        self.payment.charge(order.total())?;
        let saved = self.repo.save(&order)?;
        Ok(OrderSummary::from(saved))
    }
}
```

## Newtype Pattern for Type Safety

Prevent argument mix-ups with distinct wrapper types:

```rust
struct UserId(u64);
struct OrderId(u64);

fn get_order(user: UserId, order: OrderId) -> anyhow::Result<Order> {
    // Can't accidentally swap user and order IDs at call sites
    todo!()
}
```

## Enum State Machines

Model states as enums — make illegal states unrepresentable:

```rust
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Failed { reason: String, retries: u32 },
}

fn handle(state: &ConnectionState) {
    match state {
        ConnectionState::Disconnected => connect(),
        ConnectionState::Connecting { attempt } if *attempt > 3 => abort(),
        ConnectionState::Connecting { .. } => wait(),
        ConnectionState::Connected { session_id } => use_session(session_id),
        ConnectionState::Failed { retries, .. } if *retries < 5 => retry(),
        ConnectionState::Failed { reason, .. } => log_failure(reason),
    }
}
```

Always match exhaustively — no wildcard `_` for business-critical enums.

## Builder Pattern

Use for structs with many optional parameters:

```rust
pub struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
}

impl ServerConfig {
    pub fn builder(host: impl Into<String>, port: u16) -> ServerConfigBuilder {
        ServerConfigBuilder {
            host: host.into(),
            port,
            max_connections: 100,
        }
    }
}

pub struct ServerConfigBuilder {
    host: String,
    port: u16,
    max_connections: usize,
}

impl ServerConfigBuilder {
    pub fn max_connections(mut self, n: usize) -> Self {
        self.max_connections = n;
        self
    }

    pub fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host,
            port: self.port,
            max_connections: self.max_connections,
        }
    }
}
```

## Sealed Traits for Extensibility Control

Use a private module to seal a trait, preventing external implementations:

```rust
mod private {
    pub trait Sealed {}
}

pub trait Format: private::Sealed {
    fn encode(&self, data: &[u8]) -> Vec<u8>;
}

pub struct Json;
impl private::Sealed for Json {}
impl Format for Json {
    fn encode(&self, data: &[u8]) -> Vec<u8> { todo!() }
}
```

## API Response Envelope

Consistent API responses using a generic enum:

```rust
#[derive(Debug, serde::Serialize)]
#[serde(tag = "status")]
pub enum ApiResponse<T: serde::Serialize> {
    #[serde(rename = "ok")]
    Ok { data: T },
    #[serde(rename = "error")]
    Error { message: String },
}
```

## References

See skill: `rust-patterns` for comprehensive patterns including ownership, traits, generics, concurrency, and async.

---
paths:
  - "**/*.rs"
---
# Rust Security

> This file extends [common/security.md](common/security.md) with Rust-specific content.

## Secrets Management

- Never hardcode API keys, tokens, or credentials in source code
- Use environment variables: `std::env::var("API_KEY")`
- Fail fast if required secrets are missing at startup
- Keep `.env` files in `.gitignore`

```rust
// BAD
const API_KEY: &str = "sk-abc123...";

// GOOD — environment variable with early validation
fn load_api_key() -> anyhow::Result<String> {
    std::env::var("PAYMENT_API_KEY")
        .context("PAYMENT_API_KEY must be set")
}
```

## SQL Injection Prevention

- Always use parameterized queries — never format user input into SQL strings
- Use query builder or ORM (sqlx, diesel, sea-orm) with bind parameters

```rust
// BAD — SQL injection via format string
let query = format!("SELECT * FROM users WHERE name = '{name}'");
sqlx::query(&query).fetch_one(&pool).await?;

// GOOD — parameterized query with sqlx
// Placeholder syntax varies by backend: Postgres: $1  |  MySQL: ?  |  SQLite: $1
sqlx::query("SELECT * FROM users WHERE name = $1")
    .bind(&name)
    .fetch_one(&pool)
    .await?;
```

## Input Validation

- Validate all user input at system boundaries before processing
- Use the type system to enforce invariants (newtype pattern)
- Parse, don't validate — convert unstructured data to typed structs at the boundary
- Reject invalid input with clear error messages

```rust
// Parse, don't validate — invalid states are unrepresentable
pub struct Email(String);

impl Email {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let trimmed = input.trim();
        let at_pos = trimmed.find('@')
            .filter(|&p| p > 0 && p < trimmed.len() - 1)
            .ok_or_else(|| ValidationError::InvalidEmail(input.to_string()))?;
        let domain = &trimmed[at_pos + 1..];
        if trimmed.len() > 254 || !domain.contains('.') {
            return Err(ValidationError::InvalidEmail(input.to_string()));
        }
        // For production use, prefer a validated email crate (e.g., `email_address`)
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

## Unsafe Code

- Minimize `unsafe` blocks — prefer safe abstractions
- Every `unsafe` block must have a `// SAFETY:` comment explaining the invariant
- Never use `unsafe` to bypass the borrow checker for convenience
- Audit all `unsafe` code during review — it is a red flag without justification
- Prefer `safe` FFI wrappers around C libraries

```rust
// GOOD — safety comment documents ALL required invariants
let widget: &Widget = {
    // SAFETY: `ptr` is non-null, aligned, points to an initialized Widget,
    // and no mutable references or mutations exist for its lifetime.
    unsafe { &*ptr }
};

// BAD — no safety justification
unsafe { &*ptr }
```

## Dependency Security

- Run `cargo audit` to scan for known CVEs in dependencies
- Run `cargo deny check` for license and advisory compliance
- Use `cargo tree` to audit transitive dependencies
- Keep dependencies updated — set up Dependabot or Renovate
- Minimize dependency count — evaluate before adding new crates

```bash
# Security audit
cargo audit

# Deny advisories, duplicate versions, and restricted licenses
cargo deny check

# Inspect dependency tree
cargo tree
cargo tree -d  # Show duplicates only
```

## Error Messages

- Never expose internal paths, stack traces, or database errors in API responses
- Log detailed errors server-side; return generic messages to clients
- Use `tracing` or `log` for structured server-side logging

```rust
// Map errors to appropriate status codes and generic messages
// (Example uses axum; adapt the response type to your framework)
match order_service.find_by_id(id) {
    Ok(order) => Ok((StatusCode::OK, Json(order))),
    Err(ServiceError::NotFound(_)) => {
        tracing::info!(order_id = id, "order not found");
        Err((StatusCode::NOT_FOUND, "Resource not found"))
    }
    Err(e) => {
        tracing::error!(order_id = id, error = %e, "unexpected error");
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))
    }
}
```

## References

See skill: `rust-patterns` for unsafe code guidelines and ownership patterns.
See skill: `security-review` for general security checklists.


