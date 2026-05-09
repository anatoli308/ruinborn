# Reference vs Source — Gap Analysis

Vergleich des aktuellen Ruinborn-Setups gegen die C++-Referenz-Engine
(`steam-main/source_server` + `source_client`), aus der der ursprüngliche
Architektur-Footprint stammt. Ziel: ehrlicher Stand, was **schon abgedeckt** ist,
was **fehlt**, und was in welcher Reihenfolge sinnvoll als nächstes kommt.

> Server-autoritativer ARPG-Loop steht. Die offenen Lücken sind primär
> *MMO-Layer* (Party, Chat, Guild, Vendor) und *Polish* (Particle, CastBar).

---

## 1 · Vorhanden — Direktes Mapping

| C++ Subsystem                     | Ruinborn-Pendant                                                       |
| --------------------------------- | ---------------------------------------------------------------------- |
| `Combat/CombatFormulas`           | `combat.rs` + `damage.rs` (6 Damage-Types, Resists clamp 75 %)         |
| `Combat/CooldownManager`          | `skills.rs` Cooldown-Map pro Skill                                     |
| `Combat/SpellCaster`              | `skills::cast_skill` (DamageInstance-Build, Targeting)                 |
| `Combat/AuraSystem` (teilweise)   | `PlayerState.active_buffs` (HashMap) — *flacher als Original*          |
| `Systems/InventorySystem`         | `items.rs` (Bags, Equipment, Stacking)                                 |
| `Systems/EquipmentSystem`         | `items.rs` Paperdoll + Stat-Recompute                                  |
| `Systems/LootSystem`              | `combat.rs::LootDrop` + `skills.rs::killed_loot`                       |
| `Systems/ExperienceSystem`        | `progression.rs` (Curve, Level-Up, Stat-Punkte)                        |
| `Systems/QuestManager` (flach)    | `market.rs::mission_board` — *kein QuestLog mit Stages*                |
| `Systems/TradeSystem`             | `market.rs` Buy-/Sell-Orders an Trading-Posts                          |
| `Database/AsyncSaver`             | `db_sea.rs::SaveTask` Channel → Tokio-Worker                           |
| `Network/PacketRouter`            | `server/main.rs` `match ClientMessage`                                 |
| `Network/CharacterHandlers`       | gleiche `match`-Arme + Validation                                      |
| `AI/NpcAI`                        | `ai/goap/` + `ai/boids.rs` + `combat::tick_enemies`                    |
| `World/Entity` + `World/Player`   | `combat::Enemy`, `market::PlayerState`                                 |
| `World/Map`                       | `world.rs` Zone-Catalogue                                              |
| `Systems/NpcSpawner`              | `combat::spawn_pack` + `maintain_population` pro Zone                  |
| `Systems/AffixSystem`             | `items.rs::Affix` + Roll-Tabellen pro Rarity                           |
| `Client/Inventory + Equipment UI` | `Inventory.tsx`, `InventoryWindow.tsx`, `CharacterView.tsx`            |
| `Client/SkillTree UI`             | `SkillTreePanel.tsx`                                                   |
| `Client/Minimap`                  | `Minimap.tsx`                                                          |
| `Client/HUD + Portrait`           | `HUD.tsx`, `PortraitBar.tsx`, `ActionBar.tsx`, `MouseSkillBar.tsx`     |

---

## 2 · Fehlend — `was fehlt mir alles`

### 2a. Combat / AI

| C++ Komponente                      | Status     | Anmerkung                                           |
| ----------------------------------- | ---------- | --------------------------------------------------- |
| `AI/ThreatManager`                  | ❌ fehlt   | Aktuell nur `nearest_player`. Keine Aggro-Liste.    |
| `Combat/AuraSystem` (vollständig)   | ⚠️ partial | Buffs ja, **Debuffs auf Enemies** noch nicht echt.  |

### 2b. Systeme / Sozial

| C++ Komponente             | Status   | Anmerkung                                          |
| -------------------------- | -------- | -------------------------------------------------- |
| `Systems/PartySystem`      | ❌ fehlt | Kein Shared XP, kein Group-Loot.                   |
| `Systems/GuildSystem`      | ❌ fehlt | Keine Guilds / Guild-Bank / Roster.                |
| `Systems/ChatSystem`       | ❌ fehlt | Keine Channels, keine Whisper, kein /say.          |
| `Systems/DuelSystem`       | ❌ fehlt | Kein PvP-Toggle.                                   |
| `Systems/BankSystem`       | ❌ fehlt | Keine persistente Bank-Storage UI.                 |
| `Systems/VendorSystem`     | ❌ fehlt | Nur Trading-Posts (Markt-Orders), keine NPC-Shops. |
| `Systems/GossipSystem`     | ❌ fehlt | Keine NPC-Dialog-Trees / `DialogPanel`.            |
| `Systems/QuestManager` v2  | ❌ fehlt | Mehrstufige Quests, QuestLog-UI, Reward-Pipeline.  |

### 2c. Account / Network

| C++ Komponente                    | Status   | Anmerkung                                              |
| --------------------------------- | -------- | ------------------------------------------------------ |
| `Network/Session + SessionManager`| ❌ fehlt | Kein Login/Auth — Player-Id ist heute Session-Token.   |
| `Database/AccountDb`              | ❌ fehlt | Kein Account-Layer (1 Account → N Chars).              |
| `Shared/Md5` / Pw-Hashing         | ❌ fehlt | Folge aus „kein Auth".                                 |

### 2d. Client / UX-Polish

| C++ Komponente              | Status   | Anmerkung                                              |
| --------------------------- | -------- | ------------------------------------------------------ |
| `Client/CastBar`            | ❌ fehlt | Skills sind aktuell instant + cooldown, kein Cast-Time |
| `Client/ChatBubble`         | ❌ fehlt | Folge aus „kein Chat".                                 |
| `Client/ContextMenu`        | ❌ fehlt | Kein Right-Click-Menü auf Units (Inspect / Trade /…).  |
| `Client/ParticleSystem`     | ❌ fehlt | Skills haben heute keine Partikel-Effekte.             |
| `Client/CharacterCreation`  | ⚠️ partial | Nur ClassSelectModal, kein Name/Appearance-Setup.    |

### 2e. Editoren — *bewusst weggelassen*

`MapEditor`, `DbTemplateEditor`, `LootDbEditor`, `GossipEditor` aus dem
C++-Original werden **nicht** als In-Game-Tools nachgebaut. Wir authoren
alles über JSON-Files (`zones.json`, `enemies.json`, `goap/agents.json`) +
Code-Reload — sauberer Workflow für ein Solo-Projekt.

### 2f. Map / Streaming

| C++ Komponente              | Status   | Anmerkung                                                  |
| --------------------------- | -------- | ---------------------------------------------------------- |
| `Shared/MapCellT + MapLogic`| ❌ fehlt | Wir nutzen flache `bounds`-Rectangles, kein Cell-Streaming.|

---

## 3 · Empfohlene Reihenfolge — *next up*

Reihenfolge optimiert für **maximale Polish-Wirkung pro Aufwand** auf dem
Combat-Loot-Loop, **bevor** der MMO-Layer aufgemacht wird (siehe deine eigene
Strategie-Note).

### Phase 7 — Combat-Polish (visueller Wow-Effekt)

1. **ParticleSystem** (Client only)
   Pro Skill ein `effect_id`, gerendert via R3F. Größter
   sichtbarer Sprung für „fühlt sich gut an" — billig, keine Server-Arbeit.
2. **CastBar**
   `SkillDef.cast_ms` Feld, Server stoppt Bewegung während Cast, Client
   rendert Bar. Macht Skills wertiger als „instant + Cooldown".
3. **AuraSystem** ausweiten
   `Enemy.debuffs: HashMap<DebuffId, DebuffInstance>`. Schaltet später
   Curses (Necro), Slow (Cold), Stun-Status etc. frei.

### Phase 8 — Combat-Tiefe

4. **ThreatManager**
   Pro Enemy `threat: HashMap<PlayerId, f32>`, Aggro-Decay. Vorbereitung
   für Tank/Heal-Trinity und Group-Play.
5. **Boss-Encounters**
   Konfig-Eintrag `is_boss: true` + Phase-Skripte (z. B. Andariel im
   Catacombs L4). Belohnt das D2-Layout.

### Phase 9 — MMO-Layer (sobald Combat-Loop „klebrig" ist)

6. **PartySystem** + Shared XP / Group-Loot
7. **VendorSystem** + Gold-Sinks (Repair, Identify, Buy-Backs)
8. **QuestManager v2** mit `PlayerQuestLog`, mehrstufigen Quests,
   `QuestOffer`/`QuestComplete`/`QuestRewards`
9. **ChatSystem** (Channel + Whisper + /trade)
10. **GuildSystem** (Roster, Guild-Bank, Guild-Chat)
11. **AccountSystem** + Login (`Session`, `AccountDb`, Md5/Argon2)

### Phase 10 — Skalierung

12. **Map-Cell-Streaming** (`MapCellT`-Äquivalent)
13. **AOI-Broadcasting** (nur Spieler in Radius bekommen Snapshot-Diffs) —
    siehe [NETWORKING.md](NETWORKING.md) Roadmap.
14. **DuelSystem** + PvP-Zonen.

---

## 4 · Strategischer Take

Was wir **sehr gut** haben verglichen mit dem C++-Original:

- Reine Logik in einer No-I/O-Crate (`ruinborn-game`) — testbarer als das
  C++-Original, wo Logik und Network teils verzahnt sind.
- GOAP statt fest verdrahteter NPC-State-Machines — datengetrieben,
  pro-Archetype JSON-konfiguriert.
- Datengetriebene Zonen — Akt 2 ist nur ein JSON-Eintrag entfernt.
- WebSocket-Snapshots in JSON statt proprietärem Binär-Protokoll —
  einfacher debugbar, klarer Migrationspfad zu CBOR/Bincode.

Was wir **bewusst nicht** kopieren:

- In-Game-Editoren — JSON + Rebuild ist für ein Solo-Projekt ehrlicher.
- DirectX-Renderer — R3F/Three.js liefert genug ARPG-Look ohne eigenes
  Rendering-Stack.
- Md5 — wenn Auth kommt, dann Argon2 / scrypt, nicht Md5.

Damit ist die Lücken-Liste in 3 ehrliche Buckets sortiert: **Polish**,
**Tiefe**, **MMO-Skalierung**. Der Combat-Loot-Core ist **nicht** der
Engpass — der Engpass ist „fühlt sich der Skill geil an?".
