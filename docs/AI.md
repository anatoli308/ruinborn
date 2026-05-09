# AI — GOAP + Boids

Stand: Phasen 4 + 5 abgeschlossen. Beide Layer leben in
`crates/ruinborn-game/src/ai/`.

```
ai/
├── mod.rs
├── boids.rs       # Phase 5 — Reynolds-Flocking
└── goap/
    ├── mod.rs
    ├── action.rs       # Action + ActionState (laufende Ausführung)
    ├── condition.rs    # Condition (precondition / goal / sensor-driven)
    ├── config.rs       # JSON-Schema (Goal/Action/initial_world)
    ├── goal.rs         # Goal (Liste von Conditions + Weight)
    ├── resolver.rs     # A*-Planner über (WorldState × Goal)
    ├── runtime.rs      # AgentRuntime — Per-Enemy Plan + WorldState
    └── world_state.rs  # WorldKey/SenseValue Map
```

JSON-Configs liegen in `crates/ruinborn-game/data/goap/agents.json`
und werden zur Compile-Zeit via `include_str!` gebundelt.

---

## 1 · Architektur

```
EnemyArchetype.ai = AiKind
        │
        ├── SimpleChase  ─→ legacy direkter Chase + Attack
        │
        └── Goap         ─→ AgentRuntime
                              ├── plan: Vec<ActionId>      (von resolver.rs)
                              ├── state: WorldState
                              └── current_action_state
```

- Pro `Enemy` existiert ein `AgentRuntime` *nur*, wenn ein Match in
  `agents.json` existiert. Andernfalls **fallback** auf `SimpleChase`.
- Sensoren laufen vor dem Planner und schreiben den `WorldState`
  (`has_target`, `in_attack_range`, `target_dead`, `hp_percent`, …).
- Planner läuft, sobald Plan leer / invalid ist. A\*-Heuristik = Anzahl
  noch offener Goal-Conditions.
- Pro Tick führt das Runtime genau **eine Action** aus. Behaviour-Strings
  (`acquire_nearest_player`, `move_to_target`, `melee_attack`, `wander`,
  `flee`) werden in `runtime.rs` auf konkrete Combat-Calls gemappt.

---

## 2 · WorldState & Conditions

`WorldKey` = `String` Key, `SenseValue` = `Bool(bool)` | `Number(f32)` |
`Target(EntityId)` | `None`.

`Condition` Operatoren:

| Op                | Bedeutung                  |
| ----------------- | -------------------------- |
| `equal`           | exakt gleich               |
| `not_equal`       | ungleich                   |
| `greater_than`    | nur Number                 |
| `less_than`       | nur Number                 |
| `greater_or_equal`| nur Number                 |
| `less_or_equal`   | nur Number                 |

Effekt-Operatoren:

| Op         | Bedeutung                       |
| ---------- | ------------------------------- |
| `set`      | Wert direkt setzen              |
| `increase` | Number += value                 |
| `decrease` | Number -= value                 |

---

## 3 · Agent-Authoring

```jsonc
{
  "id": "zombie",                       // muss zu enemies.json passen
  "initial_world": { "has_target": false, "target_dead": false },
  "goals": [
    { "id": "kill_target", "weight": 1.0,
      "conditions": [{ "key": "target_dead", "op": "equal", "value": true }] }
  ],
  "actions": [
    { "id": "acquire_target", "behaviour": "acquire_nearest_player",
      "cost": 1.0,
      "preconditions": [{ "key": "has_target", "op": "equal", "value": false }],
      "effects":       [{ "key": "has_target", "op": "set",   "value": true  }] },
    { "id": "chase_target", "behaviour": "move_to_target",
      "target_key": "current_target", "in_range": 1.5, "cost": 2.0,
      "preconditions": [{ "key": "has_target",       "op": "equal", "value": true  }],
      "effects":       [{ "key": "in_attack_range",  "op": "set",   "value": true  }] },
    { "id": "melee_attack", "behaviour": "melee_attack",
      "target_key": "current_target", "in_range": 1.5,
      "ticks_to_perform": 30, "cost": 1.0,
      "preconditions": [
        { "key": "has_target",      "op": "equal", "value": true },
        { "key": "in_attack_range", "op": "equal", "value": true }
      ],
      "effects": [{ "key": "target_dead", "op": "set", "value": true }] }
  ]
}
```

Workflow: JSON editieren → `cargo check -p ruinborn-game` → Server neu starten.
Da `include_str!` zur Compile-Zeit greift, ist ein Rebuild Pflicht.

---

## 4 · Boids-Steering (Phase 5)

`ai/boids.rs` ergänzt `move_to_target` um glaubhaftes Schwarmverhalten.

Drei Reynolds-Kräfte pro Enemy, summiert und auf den Movement-Vektor addiert:

| Kraft          | Wirkung                                       |
| -------------- | --------------------------------------------- |
| **Cohesion**   | Zug zum Schwerpunkt der Nachbarn              |
| **Separation** | Repulsion bei zu enger Distanz                |
| **Alignment**  | Geschwindigkeit an Schwarmmittelwert anpassen |

Implementierungsdetails:

- Vor dem Tick wird ein `Vec<BoidSample>` gesnapshottet (`id`, `pos`, `vel`,
  `zone`). So gibt es keine `&mut`-Konflikte beim Iterieren.
- Cross-Zone-Filter: Enemies aus anderer `ZoneId` werden ignoriert (kein
  Pulling über Zonengrenzen).
- Gewichte sind aktuell hartkodiert in `boids.rs` (Cohesion 0.05,
  Separation 0.5, Alignment 0.1, Neighbour-Radius 3.0). Sind sie stabil,
  wandern sie später in `EnemyArchetype` als optionale Felder.

---

## 5 · Dispatch-Pfad

```rust
// combat.rs::tick_enemies (Pseudocode)
match archetype.ai {
    AiKind::SimpleChase => simple_chase_tick(enemy, players),
    AiKind::Goap        => match goap::runtime::tick(enemy, world_view) {
        Ok(_)  => {}
        Err(_) => simple_chase_tick(enemy, players), // safety fallback
    }
}
```

Jeder GOAP-Move-Step wird durch `boids::steer(enemy, neighbours)` korrigiert,
bevor er in `enemy.position` geschrieben wird.

---

## 6 · Tests

`cargo test -p ruinborn-game` deckt ab:

- `goap::resolver` — Plan wird gefunden / Plan-Länge minimal / unmögliches
  Goal gibt `None`.
- `goap::runtime` — Sensoren befüllen WorldState korrekt, Action-Wechsel
  bei Replan.
- `boids` — Separation überwiegt bei sehr kurzer Distanz; Cross-Zone-Boids
  beeinflussen sich nicht.

---

## 7 · Roadmap (AI)

- Threat-Manager (statt nearest-player) — siehe
  [REFERENCE_VS_SOURCE.md](REFERENCE_VS_SOURCE.md).
- Per-Archetype-Override für Boid-Gewichte.
- Hot-Reload der `agents.json` ohne Rebuild (über `std::fs` + DevServer-Flag).
- Group-Goals (Pack-Surround, Flee-Together) via shared blackboard.
