# Damage Model & Skills (Phase 1–3)

Stand: Phase 3 abgeschlossen, `cargo check --workspace` und `tsc --noEmit` grün.

Dieses Dokument beschreibt das vollständige Schadens- und Skill-System, das in
drei Phasen gebaut wurde — von der reinen Klassen-/Skill-Datenstruktur über die
UI-Anbindung bis zum typed Damage Pipeline mit Resistenzen und DoTs.

---

## Architektur-Überblick

```
┌─────────────────────────────────────────────────────────────┐
│  ruinborn-game  (pure logic, server-authoritative)         │
│                                                             │
│  damage.rs   ── DamageType, DamageTag, DamageInstance,      │
│                 Resistances, DotInstance, tick_dots()       │
│  combat.rs   ── Enemy + EnemyHit, tick_enemies, deal_damage │
│  skills.rs   ── SkillDef-Katalog, cast_skill                │
│  classes.rs  ── ClassId, Klassen-Definitionen               │
│  market.rs   ── PlayerState, advance_tick (DoT-Ticks)       │
└─────────────────────────────────────────────────────────────┘
            │ serde JSON                 ▲ Tauri events
            ▼                            │
┌─────────────────────────────────────────────────────────────┐
│  ruinborn-server  (Tauri 2 + SeaORM/Postgres)              │
│                                                             │
│  entity/player.rs ── JSONB-Spalten resistances, dots,       │
│                       allocated_skills, skill_cooldowns,    │
│                       active_buffs, class_id, …             │
│  db_sea.rs        ── ALTER TABLE Migration + Load/Save      │
└─────────────────────────────────────────────────────────────┘
            │ "game-state" snapshot/delta
            ▼
┌─────────────────────────────────────────────────────────────┐
│  Frontend (React + Zustand, read-only mirror)               │
│                                                             │
│  types/index.ts            ── DamageType/Tag, Resistances…  │
│  data/classes.ts           ── SKILL_CATALOG (gespiegelt)    │
│  store/gameStore.ts        ── ServerPlayer ↔ GameStore      │
│  components/ui/...         ── ClassSelectModal,             │
│                               SkillTreePanel, HUD-Hint      │
│  components/world/Player   ── Hotkey [K] → toggleSkillTree  │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1 — Klassen & Skill-Tree (Backend)

### Klassen

`crates/ruinborn-game/src/classes.rs`

```rust
pub enum ClassId { Barbarian, Sorceress, Necromancer }
```

Pro Klasse:

| Klasse        | Icon | Base Stats (Str/Dex/Vit/Eng) | Starter-Skill   |
| ------------- | ---- | ---------------------------- | --------------- |
| Barbar        | 🪓   | 30 / 20 / 25 / 10            | `bash`          |
| Zauberin      | 🔮   | 10 / 25 / 10 / 35            | `fireball`      |
| Totenbeschwörer | 💀 | 15 / 25 / 15 / 25            | `bone_spear`    |

`PlayerState` bekam `class_id: Option<ClassId>`, `allocated_skills: HashMap`,
`unspent_skill_points: u32`, `skill_cooldowns: HashMap`, `active_buffs:
HashMap`. Beim Level-Up wird 1 Skill-Punkt vergeben.

### Skill-Definitionen (Phase 1, ohne Damage Types)

`crates/ruinborn-game/src/skills.rs::skill_catalog()` — 9 Skills, jeweils mit
`mana_cost`, `cooldown_ticks`, `range`, `requires_level`, `effect`. Effekte in
Phase 1: `DirectDamage`, `AoeAround`, `Teleport`, `SelfBuff`, `Placeholder`.

### Tauri-Commands

- `select_class(class_id)`
- `allocate_skill(skill_id)`
- `cast_skill(skill_id, target_enemy_id?, tx?, tz?)`

Cooldowns werden je Tick im `advance_tick` heruntergezählt.

### DB-Migration

`db_sea.rs::ensure_schema` erweiterte `players` per `ALTER TABLE … ADD COLUMN
IF NOT EXISTS …` um:

```sql
class_id            TEXT,
allocated_skills    JSONB NOT NULL DEFAULT '{}'::jsonb,
unspent_skill_points INT  NOT NULL DEFAULT 0,
skill_cooldowns     JSONB NOT NULL DEFAULT '{}'::jsonb,
active_buffs        JSONB NOT NULL DEFAULT '{}'::jsonb
```

Load/Save dekodieren über `serde_json::from_value(...).unwrap_or_default()` und
`serde_json::to_value(...).unwrap_or(default)`.

---

## Phase 2 — Frontend Wiring (Klassenwahl + Skill-Tree-UI)

### Types & Catalog (gespiegelt)

`src/types/index.ts`:

```ts
export type ClassId = "barbarian" | "sorceress" | "necromancer";
export interface ClassInfo { id, name, tagline, icon, baseStats, starterSkills }
export type SkillEffectKind = "direct_damage" | "aoe_around" | "teleport"
                            | "self_buff" | "placeholder";
export interface SkillDef { id, name, classId, manaCost, cooldownTicks,
                            range, requiresLevel, effect, description }
```

`src/data/classes.ts` enthält `CLASS_CATALOG` und `SKILL_CATALOG` als 1:1
Spiegel des Rust-Katalogs.

### Store (`src/store/gameStore.ts`)

`ServerPlayer` und `GameStore` bekamen:

```ts
classId: ClassId | null;
allocatedSkills: Record<string, number>;
unspentSkillPoints: number;
skillCooldowns: Record<string, number>;
activeBuffs: Record<string, number>;
skillTreeOpen: boolean;
```

Plus Actions:

- `sendSelectClass(classId)`
- `sendAllocateSkill(skillId)`
- `sendCastSkill(skillId, targetEnemyId, tx, tz)`
- `setSkillTreeOpen`, `toggleSkillTree`

`mapSnapshot` und `mapDelta` mappen die snake_case-Felder mit Safe-Defaults.

### Components

- `src/components/ui/ClassSelectModal.tsx` — Modal beim ersten Login, ruft
  `sendSelectClass()`.
- `src/components/ui/SkillTreePanel.tsx` — Skill-Tree, Skill-Punkte vergeben,
  Casten via Nearest-Enemy-Targeting.
- `src/components/ui/HUD.tsx` — `<kbd>K</kbd> Skills`-Hinweis.
- `src/App.tsx` — beide Panels in `<div className="hud-layer">` gemountet.
- `src/components/world/Player.tsx` — Hotkey `KeyK` ruft `toggleSkillTree()`.

---

## Phase 3 — Damage Model (Types, Resistenzen, DoTs)

Neuer Kern-Modul: `crates/ruinborn-game/src/damage.rs`.

### Damage Types

```rust
#[derive(serde, snake_case)]
pub enum DamageType {
    Physical, Fire, Cold, Lightning, Poison, Magical,
}
```

### Damage Tags

Frei kombinierbar (Mehrfach-Tags pro Schaden möglich, z. B. „spell + ranged").

```rust
pub enum DamageTag { Melee, Ranged, Spell, Summoning, Trap }
```

### `DamageInstance`

```rust
pub struct DamageInstance {
    pub amount: f64,
    pub damage_type: DamageType,
    pub tags: Vec<DamageTag>,
}
```

Helpers: `DamageInstance::new(...)`, `::physical_melee(amount)`,
`.has_tag(tag)`.

### `Resistances`

```rust
pub struct Resistances {
    pub physical: f64, pub fire: f64, pub cold: f64,
    pub lightning: f64, pub poison: f64, pub magical: f64,
}
```

- `MAX_RESIST = 75.0` — Werte über 75 % werden beim Lookup geclamped.
- Negative Resistenzen erlaubt (Verwundbarkeit, z. B. `-25` Feuer beim Zombie).
- `apply(&DamageInstance) -> f64` rechnet `amount * (1 - resist/100)` und
  clampt nach unten auf 0.

### `DotInstance`

```rust
pub struct DotInstance {
    pub damage_type: DamageType,
    pub damage_per_tick: f64,
    pub ticks_remaining: u32,
    pub tags: Vec<DamageTag>,
}
```

`DotInstance::poison(dps, ticks)` — Convenience-Konstruktor mit `[Spell, Trap]`.

### `tick_dots(...)`

```rust
pub fn tick_dots(dots: &mut Vec<DotInstance>, resist: &Resistances) -> f64
```

- Dekrementiert `ticks_remaining`.
- Behält nur lebende DoTs (`retain`).
- Liefert post-resist-Summe als `f64` zurück (Caller rechnet auf `hp` an).

### Enemy-Integration

`Enemy` (in `combat.rs`) hat jetzt:

```rust
#[serde(default)] pub resistances: Resistances,
#[serde(default)] pub dots: Vec<DotInstance>,
```

`spawn_enemy` setzt `resistances = kind.resistances()`, `dots = vec![]`.

Per Enemy-Kind:

| Kind        | Schadens-Typ | Poison on hit       | Resistenzen                          |
| ----------- | ------------ | ------------------- | ------------------------------------ |
| Zombie      | Physical     | 1.5 dps × 10 ticks  | 75 poison, -25 fire                  |
| Skeleton    | Physical     | —                   | 25 phys / 50 cold / 100 poison / -25 lightning |
| FallenOne   | Fire         | —                   | 75 fire / -50 cold / 25 magical      |

`deal_damage_to_enemy` Signatur geändert auf `damage: DamageInstance` —
Resistenzen werden intern angewendet.

`tick_enemies` liefert jetzt `Vec<EnemyHit>`:

```rust
pub struct EnemyHit {
    pub player_id: String,
    pub damage: DamageInstance,
    pub poison_dot: Option<DotInstance>,
}
```

### Player-Integration

`PlayerState` (in `market.rs`) bekam:

```rust
#[serde(default)] pub resistances: Resistances,
#[serde(default)] pub dots: Vec<DotInstance>,
```

`advance_tick` flow pro Tick:

1. `tick_enemies` → `Vec<EnemyHit>` einsammeln.
2. Pro Hit: `player.resistances.apply(&hit.damage)` → HP abziehen.
   Falls `hit.poison_dot` und nicht bereits dominanter Poison-DoT vorhanden →
   anhängen / ersetzen (Replace-Regel: neuer Total-Damage ≥ existierender).
3. `tick_dots(&mut player.dots, &player.resistances)` — bei Tod:
   `notification = "💀 Du bist an Gift gestorben!"`.
4. Pro Enemy: `tick_dots(&mut enemy.dots, &enemy.resistances)` — bei Tod:
   `state = Dead`, `despawn_in` setzen.

### Skills mit Damage-Types

`SkillDef` erweitert um:

```rust
pub damage_type: Option<DamageType>,
pub tags: &'static [DamageTag],
```

Aktualisierter Katalog:

| Skill ID         | Klasse         | Effect            | DamageType | Tags                  |
| ---------------- | -------------- | ----------------- | ---------- | --------------------- |
| `bash`           | Barbar         | DirectDamage      | Physical   | [Melee]               |
| `cleave`         | Barbar         | AoeAround         | Physical   | [Melee]               |
| `battle_cry`     | Barbar         | SelfBuff          | None       | []                    |
| `fireball`       | Zauberin       | DirectDamage      | Fire       | [Spell, Ranged]       |
| `frost_nova`     | Zauberin       | AoeAround         | Cold       | [Spell]               |
| `teleport`       | Zauberin       | Teleport          | None       | [Spell]               |
| `bone_spear`     | Necromancer    | DirectDamage      | Magical    | [Spell, Ranged]       |
| `raise_skeleton` | Necromancer    | Placeholder       | None       | [Summoning]           |
| `amplify_damage` | Necromancer    | DamageOverTime    | Poison     | [Spell, Trap]         |

`amplify_damage` heißt jetzt **„Giftwolke"** — 4 dps × 20 Ticks, Range 8.

`SkillEffect` hat zusätzliche Variante:

```rust
DamageOverTime { dps: f64, ticks: u32 }
```

`cast_skill` baut für `DirectDamage`/`AoeAround` ein `DamageInstance` aus
`def.damage_type` und `def.tags`. Für `DamageOverTime`: Range-Check + Helper:

```rust
pub fn apply_dot_to_enemy(enemies, id, dot)
```

Replace-Regel: ersetzt einen bestehenden DoT desselben Typs nur, wenn der neue
Gesamtschaden (`dps × ticks`) mindestens so groß ist.

### DB-Migration Phase 3

`entity/player.rs` zwei neue Spalten:

```rust
#[sea_orm(column_type = "JsonBinary")] pub resistances: serde_json::Value,
#[sea_orm(column_type = "JsonBinary")] pub dots:        serde_json::Value,
```

`db_sea.rs::ensure_schema` ergänzt:

```sql
ADD COLUMN IF NOT EXISTS resistances JSONB NOT NULL DEFAULT '{}'::jsonb,
ADD COLUMN IF NOT EXISTS dots        JSONB NOT NULL DEFAULT '[]'::jsonb
```

Load:

```rust
let resistances: Resistances =
    serde_json::from_value(pm.resistances).unwrap_or_default();
let dots: Vec<DotInstance> =
    serde_json::from_value(pm.dots).unwrap_or_default();
```

Save: analog `to_value(...).unwrap_or(...)` + `Set(...)` im `ActiveModel`.

### Frontend-Mirror Phase 3

`src/types/index.ts`:

```ts
export type DamageType = "physical" | "fire" | "cold" | "lightning"
                       | "poison" | "magical";
export type DamageTag  = "melee" | "ranged" | "spell" | "summoning" | "trap";
export interface Resistances { physical, fire, cold, lightning, poison, magical: number }
export interface DotInstance { damage_type, damage_per_tick, ticks_remaining, tags }
export type SkillEffectKind = "direct_damage" | "aoe_around" | "damage_over_time"
                            | "teleport" | "self_buff" | "placeholder";
export interface SkillDef { …, damageType: DamageType | null, tags: DamageTag[] }
```

`src/data/classes.ts` — alle Skills tragen jetzt `damageType` + `tags`.
„Giftwolke" mit `effect: "damage_over_time"`, `damageType: "poison"`,
`tags: ["spell", "trap"]`.

`src/store/gameStore.ts` — `ServerPlayer` und `GameStore` haben `resistances`
+ `dots`, beide Mapper liefern Safe-Defaults.

`src/components/ui/SkillTreePanel.tsx` — `EFFECT_LABEL["damage_over_time"] =
"Gift-DoT"`, Targeting für DoT-Skills nutzt `nearestEnemyId()` analog zu
`direct_damage`.

---

## Persistenz-Übersicht

In `players` (Postgres) JSONB-Spalten relevant für Damage/Skills:

| Spalte                | Typ                  | Default       |
| --------------------- | -------------------- | ------------- |
| `class_id`            | TEXT (nullable)      | NULL          |
| `allocated_skills`    | JSONB Map<id, level> | `{}`          |
| `unspent_skill_points`| INT                  | 0             |
| `skill_cooldowns`     | JSONB Map<id, ticks> | `{}`          |
| `active_buffs`        | JSONB Map<id, ticks> | `{}`          |
| `resistances`         | JSONB                | `{}`          |
| `dots`                | JSONB Array          | `[]`          |

Migrations sind idempotent (`ADD COLUMN IF NOT EXISTS`), bestehende Spielstände
werden durch `serde(default)` + `unwrap_or_default()` automatisch aufgewertet.

---

## Hotkeys

| Taste | Aktion                  |
| ----- | ----------------------- |
| `C`   | Character-View          |
| `K`   | Skill-Tree              |
| `1–9` | Action-Bar              |
| `B`   | Tausch-Panel            |

---

## Verify-Status

| Check                              | Status |
| ---------------------------------- | ------ |
| `cargo check --workspace`          | ✅     |
| `cargo check` (`src-tauri`)        | ✅     |
| `npx tsc --noEmit`                 | ✅     |

---

## Offene Punkte / nächste Phasen

- **Summoning** (`raise_skeleton`): Tag und Skill liegen, Effekt ist noch
  `Placeholder`. Nächste Phase: Pets/Minions als `Enemy`-Variante mit
  `owner_player_id`.
- **Trap-DoTs**: aktuell wird der Tag nur deklarativ gesetzt; eigene
  Trap-Entity (statisch, mit AoE-Tick) noch zu bauen.
- **Magical Resist** existiert technisch, wird aber bisher nur von
  `bone_spear` erzeugt — bewusst mager gehalten bis Necromancer-Curses kommen.
- **Lightning** ist als Type angelegt, aktuell ohne Skill.
- Frontend zeigt `resistances` + aktive `dots` noch nicht im HUD an —
  Datenpfad ist da, UI-Element fehlt noch (Character-View).
