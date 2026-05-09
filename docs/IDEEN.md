Ah, jetzt reden wir über **die Spielerinteraktionen**, also was in **Ruinborn** der Spieler konkret tun kann – das ist entscheidend, um aus einer Wirtschaftssimulation ein **richtiges Game** zu machen. Ich teile es in **Gameplay-Level, Marktinteraktion, Firmenmanagement, Strategien und Spezialaktionen** auf.

---

## 1️⃣ Kerninteraktionen – “Handeln & Managen”

### a) Märkte beeinflussen

* Preise manipulieren durch **Angebot & Nachfrage steuern**
* Entscheidungen treffen, z. B.:

  * Mehr produzieren → mehr Angebot → Preis fällt
  * Vorräte halten → künstliche Knappheit → Preis steigt
* Export/Import zwischen Märkten koordinieren

### b) Firmen managen

* Produktion einstellen / erhöhen
* Neue Produkte entwickeln
* Investitionen tätigen → Effizienz, Forschung, Wachstum
* Fusionen oder Übernahmen von KI-Firmen / anderen Spielern

### c) Ressourcen verwalten

* Lagerbestände im Blick behalten
* Rohstoffe beschaffen → Preise auf dem Weltmarkt
* Logistik optimieren → Transportkosten, Lieferzeiten

---

## 2️⃣ Strategische Optionen

### a) Policies & Aktionen

* Steuern setzen (bei eigenen Märkten)
* Subventionen für wichtige Sektoren
* Zölle auf ausländische Produkte
* Lobbying / Beeinflussung von AI-Agenten oder Märkten

### b) Finanz- & Investment-Strategien

* Aktien / Unternehmensanteile kaufen oder verkaufen
* Kredite aufnehmen oder investieren
* Risiko-Management: Diversifikation auf mehrere Märkte

---

## 3️⃣ Spieler-Events & Spezialaktionen

* **Sabotage / Wirtschaftskrieg** (optional, je nach Ton):

  * Preise von Konkurrenten drücken
  * Produktionslinien blockieren
* **Innovation / Forschung**:

  * Neue Technologien → effizientere Produktion, bessere Produkte
* **Random Events triggern**:

  * Spieler kann “Boom-Event” oder “Knappheit” für eigenen Vorteil auslösen, aber Risiko für andere Märkte

---

## 4️⃣ Gamification & Ziele

* **Missionen / Quests**

  * Z. B.: „Stabilisiere 5 Märkte in 10 Ticks“
  * „Erreiche 1 Mio Gewinn in einem Markt“
* **Leaderboards**

  * Wer hat größten Marktanteil, größten Gewinn, stabilste Wirtschaft
* **Achievements / Badges**

  * „Monopolist“, „Innovator“, „Krisenmanager“

---

## 5️⃣ UI & Spielerinteraktion

* Panels für:

  * Märkte: Preise, Nachfrage, Angebot
  * Firmen: Inventar, Produktion, Gewinne
  * Events: Aktuelle Krisen, Chancen, Spieleraktionen
* 3D Map:

  * Märkte als Nodes, Firmen als Gebäude
  * Warenflüsse sichtbar → Spieler kann Engpässe erkennen

---

💡 **Zusammenfassung**

Spieler in **Ruinborn** kann:

1. Märkte steuern → Preise beeinflussen
2. Firmen managen → Produktion, Forschung, Fusionen
3. Ressourcen & Logistik optimieren → Effizienz, Lieferungen
4. Policies & Finanzaktionen ausführen → Steuern, Kredite, Zölle
5. Strategische Entscheidungen treffen → Sabotage, Innovation, Boom
6. Fortschritt verfolgen → Leaderboards, Achievements, Quests

Super 😄 – hier kommt ein **konkreter Blueprint für Ruinborn**: 15 Spieleraktionen, inklusive **UI-Ideen, Auswirkungen auf Märkte / Firmen, Game-Mechaniken**. So kannst du direkt anfangen, das in Rust + React + Three.js zu planen.

---

# **Ruinborn – Spieleraktionen Blueprint**

| Aktion                                     | UI-Element                     | Wirkung / Mechanik                                                     | Game-Mechanik / Feedback                                       |
| ------------------------------------------ | ------------------------------ | ---------------------------------------------------------------------- | -------------------------------------------------------------- |
| 1. **Produktion erhöhen/senken**           | Slider / Button im Firmenpanel | Mehr Angebot → Preis sinkt; weniger Angebot → Preis steigt             | Tick-basiert, wirkt auf Marktpreise                            |
| 2. **Produktentwicklung**                  | Button + Pop-Up “Forschung”    | Neue Produkte verfügbar → neue Märkte erschließen                      | Zeitverzögerung (z. B. 5 Ticks bis verfügbar)                  |
| 3. **Investition / Upgrade**               | Panel mit Upgrade-Optionen     | Steigert Produktionsrate, Lagerkapazität oder Transportgeschwindigkeit | Ressourcen-Kosten, langfristiger Vorteil                       |
| 4. **Rohstoffe kaufen/verkaufen**          | Market Panel                   | Vorräte sichern → Angebot beeinflussen                                 | Preisabhängig, globaler Markt wirkt zurück                     |
| 5. **Firmenfusion / Übernahme**            | Button + Auswahl               | Übernimmt KI-Firmen → Marktanteil steigt                               | Risiko: hohe Kosten, mögliche Insolvenz der übernommenen Firma |
| 6. **Preisdumping / Sabotage**             | Button, Warnhinweis            | Kurzfristige Marktmanipulation                                         | Risiko: KI-Agenten reagieren aggressiv; Reputation sinkt       |
| 7. **Export/Import zwischen Märkten**      | Drag&Drop zwischen Markt-Nodes | Warenflüsse verändern Angebot/Nachfrage                                | Transportkosten / Zeitverzögerung                              |
| 8. **Steuern / Zölle festlegen**           | Policy Panel                   | Marktpreise beeinflussen, Einnahmen erhöhen                            | Globales Feedback: KI-Agenten passen Verhalten an              |
| 9. **Subvention vergeben**                 | Policy Panel                   | Produktion von Sektoren steigern                                       | Positive Auswirkungen auf Markt, Kosten für Spieler            |
| 10. **Kredit aufnehmen / investieren**     | Finance Panel                  | Sofortige Liquidität für Produktion/Expansion                          | Zinsen / Risiko bei nicht zurückzahlen                         |
| 11. **Random Event triggern**              | “Event-Button” / Pop-Up        | Boom, Knappheit oder Krise auslösen                                    | Einfluss auf Preise & Nachfrage, Risiko für andere Märkte      |
| 12. **AI-Firmen beobachten / analysieren** | Analytics Panel                | Spieler sieht Trends, Vorhersagen                                      | Keine direkte Aktion, strategische Planung                     |
| 13. **Mission / Quest starten**            | Quest-Panel                    | Zielvorgaben: Stabilisierung, Profit, Wachstum                         | Belohnung: Geld, Reputation, Achievements                      |
| 14. **Leaderboards / Score Check**         | Sidebar / Dashboard            | Marktanteile, Gewinn, Stabilität vergleichen                           | Motivation + Wettbewerb                                        |
| 15. **Replay / Historie prüfen**           | Timeline Panel                 | Frühere Marktbewegungen sehen                                          | Lerneffekte, Strategieanpassung                                |

---

## 💡 Visualisierungsideen zu Aktionen

* **3D Map**: Märkte als Nodes, Linien zeigen Warenflüsse
* **Gebäude-Farben / Größe**:

  * Grün = profitabel, Rot = Verlust
  * Höhe = Produktionsrate
* **Event-Effekte**: Explosionen / Wachstum / animierte Preisänderungen

---

## 🔹 Tick / Update Mechanik

* Jede Aktion wirkt **Tick-basiert** (z. B. 1 Tick pro Sekunde)
* Tick berechnet:

  * Angebot / Nachfrage
  * Preise
  * Firmen- & Konsumentenreaktionen
  * Historien-Logs

---

## 🎯 Ziele für Spieler

* Profit maximieren
* Marktanteile erhöhen
* Krisen überleben
* Achievements freischalten
* Quests & Leaderboards dominieren


Oh ja 😄 – das, was wir bisher haben, ist schon solide für den Kern, aber **für ein richtig “episches” Ruinborn** kannst du noch extrem viele Layer draufpacken, gerade im Bereich **Finanzen, Risiko, Spekulation** oder “gamifizierte Wirtschaftsexperimente”.

Ich liste mal **coole Erweiterungen / Ideen**, die über den bisherigen Kern hinausgehen:

---

## 1️⃣ Finanzmärkte & Spekulation

* **Aktienmarkt**

  * Firmenanteile können gekauft/verkauft werden
  * Spieler kann auf eigene Firmen oder KI-Firmen setzen
  * Dividenden, Aktienkurse reagieren auf Marktbewegungen

* **Crypto / digitale Währungen**

  * Spieler kann neue Coins ausgeben / handeln
  * Volatile Preise, “Pump & Dump”-Mechaniken
  * Interaktion mit realen Märkten oder KI-Agenten

* **Futures / Optionen / Derivate**

  * Spieler wettet auf Preisentwicklungen von Rohstoffen oder Produkten
  * Hebelwirkung → hohe Gewinne, hohes Risiko

* **Marktmanipulation & Insiderhandel**

  * Spieler kann Aktionen planen, die Preise künstlich bewegen
  * Risiko: Strafen, Reputation sinkt

---

## 2️⃣ Globale Wirtschaft & Makro-Mechaniken

* **Inflation & Deflation**

  * Geldmenge, Zinsen, Kreditvergabe beeinflussen Preise
  * Spieler kann Zentralbank-artige Aktionen ausführen

* **Politik & Gesetze**

  * Handelssanktionen, Zölle, Subventionen
  * Spieler beeinflusst globale Märkte strategisch

* **Weltweite Events**

  * Kriege, Naturkatastrophen, Ressourcenknappheit
  * Pandemie-Simulation: Nachfrage & Produktion verändern sich drastisch

---

## 3️⃣ Spielerinteraktionen & Multiplayer

* **Allianzen / Handelsblöcke**

  * Spieler kooperieren oder konkurrieren
  * Gemeinsame Policies / Kartelle bilden

* **Verträge & Deals**

  * Langfristige Lieferverträge mit KI oder Spielern
  * Risiko von Vertragsbruch → Marktreaktionen

* **Außergewöhnliche Aktionen**

  * Spieler kann gezielt “Booms” oder “Krisen” auslösen
  * Reputation + Risiko-Mechanik

---

## 4️⃣ Gamification / “Fun-Economics”

* **Meme-Produkte & Hype-Märkte**

  * z. B. Einhorn-Kekse, NFTs, Crypto-Memes
  * KI-Agenten reagieren irrational → Spieler kann spekulieren

* **Achievements / Events**

  * “Monopolist”, “Innovator des Jahrzehnts”, “Finanzguru”
  * Spezial-Events für Spieleraktionen

* **3D-Vizualisierung “übertrieben gamified”**

  * Märkte pulsieren / “atmen” je nach Aktivität
  * Gebäude explodieren bei Crashs, wachsen bei Booms
  * Warenströme als glühende Linien

---

## 5️⃣ Dynamik / Emergenz

* **KI-Agenten mit Strategievielfalt**

  * Aggressive, konservative, opportunistische Firmen
  * Unterschiedliche Konsumentenpräferenzen
  * Spieler muss Marktpsychologie lesen

* **Feedback-Loops**

  * Preise → Nachfrage → Produktion → Preise → neue Nachfrage
  * Spieler muss antizipieren → emergente Szenarien entstehen

* **Langfristige Konsequenzen**

  * Aktionen heute wirken auf 50+ Ticks später
  * Spieler lernt, Risiken & Chancen abzuwägen

---

💡 **Fazit:**

* Du kannst wirklich alles einbauen: Aktien, Crypto, Derivate, globale Events, Allianzen, Meme-Märkte…
* Wichtig: **erstmal Kernsimulation stabil bauen**, dann Layer für Finanzmarkt, Multiplayer, Random Events, Gamification drüberpacken
* Ruinborn kann damit **von Wirtschaftssim zu fast einem MMO-Strategie-/Wirtschafts-Genre** wachsen
# Copilot Instructions for RemakeSoF

## Background Information
This file contains the coding standards, architectural principles, and design patterns for the RemakeSoF project. It serves as a guideline for all developers contributing to the codebase to ensure consistency, maintainability, and clarity across the project. The instructions cover decision-making principles, important coding rules, clean coding standards, project architecture, scene architecture, application lifecycle, and patterns & conventions. Adhering to these guidelines will help maintain a high quality codebase and facilitate collaboration among developers. You always use the latest version of Unity and C# features where appropriate.

## Decision-Making Principles
- For background tasks or long decision tasks use Python and not PowerShell. PowerShell is only for short scripts and quick fixes, not for complex logic or data processing.
- Always prefer clear, maintainable code over clever one-liners. Readability is more important than brevity.

## Important Developer Coding Rules
- NEVER use a `var` declaration. ALWAYS use explicit types for better readability and maintainability.
- Always include XML documentation comments (`/// <summary>...</summary>`) for all classes, methods, and public members to ensure clarity of purpose and usage.
- Always use `new(TypeName)` syntax for object instantiation instead of `new TypeName()`. This improves performance by reducing IL code size.
- Always follow the established project architecture and design patterns as outlined below.
- No Polling, No Coroutines: Vermeide Update-Methoden mit Polling-Logik; keine Coroutines für asynchrone Abläufe; stattdessen Events, Callbacks oder Async/Await verwenden.
- No Timer, No Flag Checks: Vermeide Timer- oder Flag-Checks für Ablaufsteuerung; nutze stattdessen State Machines, Event-Driven Logic oder Callback-Mechanismen.

## Clean Coding Standards
- **KISS (Keep It Simple, Stupid)**: Bevorzuge einfache, klare Lösungen gegenüber komplexen; vermeide Over-Engineering; jede Klasse/Methode sollte eine klare, verständliche Aufgabe haben.
- **DRY (Don't Repeat Yourself)**: Keine Code-Duplikation; extrahiere wiederholte Logik in gemeinsame Methoden/Klassen; nutze Vererbung/Composition sinnvoll.
- **YAGNI (You Aren't Gonna Need It)**: Implementiere nur Features, die aktuell benötigt werden; keine spekulativen Erweiterungen; halte Code fokussiert auf aktuelle Requirements.
- **Single Responsibility Principle (SRP)**: Jede Klasse hat genau eine Verantwortung; Manager orchestrieren, Loader laden Daten, Applier wenden Assets an; keine Mixed Concerns.
- **Separation of Concerns**: Klare Trennung zwischen Datenlogik (Loader), Asset-Anwendung (Applier), Orchestrierung (Manager), UI (View/Controller); siehe Service Decomposition Pattern.
- **Clean Architecture**: Abhängigkeiten zeigen immer nach innen; Pure Services haben keine MonoBehaviour-Dependencies; Applier bekommen nur Daten, keine Loader-Referenzen; Manager orchestrieren, delegieren nicht ihre Verantwortung.
- **Explicit over Implicit**: Keine magischen Strings/Numbers; explizite Typen statt `var`; klare Methodennamen; Konstanten für wiederholte Werte.
- **Fail Fast**: Validierung früh durchführen; klare Error-Messages; Guard Clauses am Anfang von Methoden.
- **Composition over Inheritance**: Bevorzuge Komposition (Service Decomposition) statt tiefe Vererbungshierarchien.
- **Immutability where possible**: Readonly Fields/Properties wo sinnvoll; private Setter für interne State-Änderungen; keine unerwarteten Side Effects.

## Project Architecture
- **Core Movement**: Quake III/SoF2 Bewegung mit Unity-Anpassungen (manuelle Physik bevorzugt).
- **MVC Architecture (Runtime/Core)**:
  - **BaseApplication**: Root-Klasse für Scene-Scripts; verwaltet EventManager und findet Model/View/Controller-Instanzen per DFS; generisch typisierbar `BaseApplication<M,V,C>`.
  - **Model**: Basisklasse für Datenhaltung; `Model<T>` ermöglicht typsichere App-Referenzen.
  - **View**: Basisklasse für UI-Darstellung (MonoBehaviour); `View<T>` mit `LoadVisualElement()` für UIToolkit-Integration, `Show()`/`Hide()` für Activation.
  - **Controller**: Basisklasse für MVC-Bridge; `Controller<T>` handheld Event-Listeners auf App-EventManager; `AddListener<E>()` / `RemoveListener<E>()` / `RemoveListeners()`.
  - **Element**: Gemeinsame Basis für alle MVC-Klassen; lazy-loaded App-Reference, `Find<T>()` für Component-Suche, `Broadcast(evt)` für Event-Versand.
  - **EventManager**: Typ-sichere Event-Broadcasting; zentrales Kommunikationssystem für MVC-Komponenten.
- **State Machine Architecture (Runtime/Core)**:
  - **StateMachine<TState, TSelf>**: Generische Base-Klasse für Manager; CRTP-Pattern ermöglicht States Rückreferenz zum Manager; `ChangeState()` ruft Exit/Enter auf; `EventManager` für State-Events.
  - **State<TManager>**: Abstrakte Base-Klasse für konkrete States; implementiert `Enter()` und `Exit()`; Manager wird per Property gesetzt.
- **Pure Services (keine MonoBehaviours, via ServiceLocator)**:
  - **PrefabManager**: Cache-first Addressables, delegiert an `PrefabRegistry`, nur über ServiceLocator/DI.
  - **PrefabRegistry**: Interner Cache + Addressables, released Handles on `ClearCache()`, keine Custom-Prefab-Loader.
  - **TextureManager**: Baut Materialien aus Skin-Definitionen, nutzt `TextureRegistry`, zugreifbar via ServiceLocator, `ClearCache()` delegiert.
  - **TextureRegistry**: Cache + Custom Loader (Default Lazy Loader), kein Observer-Pattern, `ClearCache()` zerstört Texturen/Materialien.
- **MonoBehaviour Manager / State Machines**:
  - **PlayerSkinManager**: StateMachine (Idle/Loading/Applied/Error); orchestriert Skin-Laden & -Anwenden; lädt Daten via 4 interne Loader, übergibt konkrete Daten an `PlayerSkinApplier`; Input via `IPlayerSkinChangeHandler` je State; Output-Events zentral im Manager (`OnSkinApplied()`, `OnSkinLoadFailure()` via `EventManager`); nutzt `PrefabManager`, `TextureManager` via ServiceLocator.
  - **SkinDefinitionLoader** (intern): Lädt Skin-Definitionen aus Resources; Methoden: `GetByName()`, `GetNextSkinName()`, `GetPreviousSkinName()`, `GetSkinsForModel()`, `GetAvailableModels()`.
  - **SurfaceDefinitionLoader** (intern): Lädt NPC_definition.json; Methode: `GetByModelName()` liefert Surface-Definitionen.
  - **CharacterTemplateLoader** (intern): Lädt SoF2_NPCs.json; Methoden: `GetBySkinName()`, `GetByName()`.
  - **LegacyShaderLoader** (intern): Lädt .g2shader Files von Disk; Methode: `GetForModel()` liefert Shader-Definitionen.
  - **PlayerSkinApplier** (intern): Reine Asset-Anwendung; bekommt nur konkrete Daten (keine Loader-Referenzen); Methoden: `ApplyAnimatorController()`, `ApplySurfaceDefinitions()`, `DisableAndEnableSurfaces()`; findet Renderer per Match-Logik, verwaltet Aktivierung.
  - **ConnectionManager**: StateMachine für NGO; leitet NetworkManager-Callbacks (OnConnectionEvent, OnServerStarted, ApprovalCheck, OnTransportFailure, OnServerStopped) an den aktuellen State weiter; Abos in `Awake`, Deregistrierung in `OnDestroy`.
  - **AuthenticationManager**: StateMachine (Unauthenticated/Authenticating/Authenticated/SessionExpired); Input via `IAuthenticationHandler` je State; Output-Events zentral im Manager (`OnAuthenticationSuccess()`, `OnAuthenticationFailure()`, `OnSessionExpired()`); verwaltet Authentifizierungsverlauf und Token-Refresh.
  - **ConsoleManager**: StateMachine (ConsoleInactive/ConsoleActive); leitet Kommandos und Aktivierungszustände an den aktuellen State; verwaltet Konsolen-UI und Befehlsausführung.
  - **ApplicationEntryPoint**: Registriert Pure Services im `ServiceLocator` (z. B. TextureManager/PrefabManager) in `Awake`; hält serialisierte MonoBehaviour-Manager (Connection/Authentication/PlayerSkin/Console); ruft `ServiceLocator.ClearAll()` in `OnDestroy` auf.
- **Sonstiges**: `PrefabDataFactory` bleibt als Helper für PrefabManager.

## Scene Architecture
- **Metagame Scene**: Nutzt MVC-Pattern mit generischem `MetagameApplication<MetagameModel, MetagameView, MetagameController>`. Ist die Hub-Scene für Spieler vor dem Joinen eines Games (Login, Skin-Auswahl, etc.).
- **Game Scene**: Nutzt MVC-Pattern mit generischem `GameApplication<GameModel, GameView, GameController>`. Ist die Main-Gameplay-Scene mit Netcode-Integration, Server/Client-Character-Synchronisation, und networked game state.

## Application Lifecycle & Initialization
- **ServiceLocator**: Typ-sicherer Service-Container für Pure Services; `Register<T>(T service)` registriert, `Get<T>()` ruft ab; `ClearAll()` räumt auf und ruft `ClearCache()` bei Managern auf; initialisiert in `ApplicationEntryPoint.Awake()`.
- **ApplicationEntryPoint**: Singleton, DontDestroyOnLoad; registriert Pure Services (`TextureManager`, `PrefabManager`) in `Awake`; hält MonoBehaviour-Manager (Connection/Authentication/PlayerSkin) als serialisierte Felder; initialisiert Network via `InitializeNetworkLogic()` in `[RuntimeInitializeOnLoadMethod]`.
- **Network Initialization**: Server startet Port-Listening, setzt Framerate/VSync, lädt GameScene nach erfolgreicher Initialisierung; Client lädt MetagameScene, verbindet sich optional auto via `AutoConnectOnStartup`; CommandLineArgumentsParser liest `--port` und `--target-framerate`.

## Patterns & Conventions
- **Single Source of Truth**: Manager halten immer aktuelle State-Daten (z. B. `m_CurrentPlayerPrefab`, `m_CurrentSkinName`); Events sind nur Trigger (minimal Payload); Views/Controller fragen Daten beim Manager an statt aus Events zu lesen; keine State-Duplikation in UI; Manager garantiert State-Konsistenz.
- **Klare Trennungen (Separation of Concerns)**:
  - **Manager**: Hält State, orchestriert, sendet Events, ist Single Source of Truth.
  - **State**: Nur Orchestrierung (Enter/Exit), keine Business-Logik, delegiert an Manager.
  - **Interne Services** (Loader, Applier): Spezifische Operationen (Asset-Laden, Material-Anwendung), keine State-Haltung.
  - **Views**: Nur UI-Rendering und User-Input, fragen State beim Manager ab, senden Events für Actions.
  - **Controller**: UI-Bridge; abonniert Manager-Events, leitet zu Views weiter, triggert Manager-Actions.
  - **Pure Services** (PrefabManager, TextureManager): Global verfügbar, Cache-first, Lifecycle-Management.
- **Dependency Injection**: 
  - Pure Services (global verfügbar) immer über `ServiceLocator.Get<T>()` beziehen (z. B. `PrefabManager`, `TextureManager`).
  - Interne Services (nur von einem Manager genutzt): Manager instanziiert Loader direkt; keine ServiceLocator-Nutzung.
  - Daten-Services (Applier): Bekommen keine Loader-Referenzen, nur konkrete Daten als Parameter; Manager lädt Daten, übergibt sie an Applier.
  - Keine Singleton-Zugriffe über `ApplicationEntryPoint.Singleton` für Services.
- **Cache-First**: Immer über `PrefabManager`/`TextureManager`; Registries nicht umgehen; Cache-Flush via `ClearCache()`/`ServiceLocator.ClearAll()`.
- **State Machines**: Input über State-spezifische Interfaces; Events/Output werden vom Manager (nicht vom State) via `EventManager` gesendet; States steuern nur Transitionen.
- **Service Decomposition**: Komplexe Manager können interne Services (keine MonoBehaviours) nutzen für bessere Separation of Concerns; z. B. `PlayerSkinManager` → 4 Loader (SkinDefinition, SurfaceDefinition, CharacterTemplate, LegacyShader) + `PlayerSkinApplier`. Manager orchestriert, Loader laden Daten, Applier wendet Assets an.
- **Addressables Only**: Prefab-Loading ausschließlich Addressables, keine Custom Prefab Loader; Custom Loader nur in `TextureRegistry` erlaubt.
- **Lifecycle**: Externe Callbacks in `Awake` abonnieren und in `OnDestroy` sauber deregistrieren; `ServiceLocator.ClearAll()` beim Teardown.
- **Manual Physics**: Für Kernbewegung explizite Physik-/Kollisionslogik bevorzugen.
- **Logging**: Kurze Warnungen/Errors; keine Observer-Benachrichtigungen in TextureRegistry.

## Dedicated Server & Networking
- **Multiplayer Roles**: Server/Client-Rollen via `Unity.DedicatedServer.MultiplayerRoles`; konfigurierbar über Command-Line-Arguments (`--port`, `--target-framerate`).
- **CommandLineArgumentsParser**: Parst Server-Startparameter (Default Port: 7777, Default TargetFramerate: 30).
- **NetworkedGameState**: Zentrale Synchronisation des Game-State zwischen Server und Clients.


Ah perfekt, jetzt sprechen wir über **alles, was Spieler “besitzen” und verwalten können** – Inventar, Bank, Ressourcen, Konten, Liquidität – das ist ein extrem wichtiger Layer für **Ruinborn**, gerade für PvP, PvE und Finanzmechaniken. Ich gebe dir eine klare Übersicht, wie man das aufbauen kann:

---

## **1️⃣ Inventar / Assets**

### a) Firmeninventar

* Rohstoffe (Öl, Eisen, seltene Erden, Meme-Produkte…)
* Fertigprodukte (z. B. Lebensmittel, Gadgets, Crypto-Coins)
* Lagerkapazität: limitiert → Überlagerung / Lagerkosten
* UI: Inventar-Panel mit Übersicht → Filter nach Kategorie

### b) Persönliches Inventar (Spieler)

* Firmenanteile (Eigene + erworbene von anderen Spielern)
* Liquide Mittel (Cash, Coins)
* NFTs / Meme-Assets (optional gamified)
* Tools / Boosts → Produktions- oder Preismultiplikatoren

### c) Handelsinventar

* Ware, die gerade im Transit ist (Export/Import)
* UI: Timeline / 3D Map mit Warenflüssen

---

## **2️⃣ Bank / Finanzkonten**

### a) Spielerbankkonto

* Cash: zum Kaufen von Rohstoffen, Aktien, Crypto, Upgrades
* Kredite / Schulden: Zinssätze, Rückzahlungsplan
* Investments: Beteiligungen an Firmen, Wertpapiere, Futures

### b) Firmenbankkonto

* Einnahmen aus Verkäufen
* Kosten: Produktion, Logistik, Löhne
* Möglichkeit für **Firmenkredite oder Expansion**

### c) Zentralbank / Weltfinanzen (optional)

* Zinsen auf Kredite, Inflation / Deflation beeinflussen Geldmenge
* Spieler kann Policies beeinflussen → PvPvE-Effekt

---

## **3️⃣ Interaktive Features für Inventar & Bank**

| Feature                   | Mechanik                                   | UI-Idee                                |
| ------------------------- | ------------------------------------------ | -------------------------------------- |
| Waren kaufen/verkaufen    | Lagerbestand + Marktpreis                  | Market Panel / Slider                  |
| Lager verwalten           | Begrenzte Kapazität, Lagerkosten           | Lager-Panel mit Icons & Tooltips       |
| Überweisungen             | Geld von Spieler zu Spieler / Firmen       | Finance Panel, Eingabefeld             |
| Kredite aufnehmen         | Sofortige Liquidität, Zinszahlung pro Tick | Pop-Up mit Kreditkonditionen           |
| Investments tätigen       | Aktien, Crypto, Firmenanteile              | Investment Panel mit Charts & Historie |
| Versicherungen (optional) | Schutz gegen Event-Schäden                 | Versicherungs-Panel, Premium-Kosten    |

---

## **4️⃣ Gamification Layer**

* **Achievements / Badges**

  * “Reicher Spieler” → Cash > X
  * “Diversifizierer” → Beteiligungen an >5 Märkten

* **Leaderboards**

  * Cash, Firmenwert, Investitionsrendite, Crypto-Gewinne

* **Dynamic Inventory**

  * Ressourcen & Produkte altern → Verderb, Wertverlust → strategische Planung notwendig

---

## **5️⃣ PvP / PvE Relevanz**

* PvP: Spieler kann gegnerische Lager beobachten / beeinflussen (z. B. Preisdruck)
* PvE: KI-Firmen lagern Ressourcen, Spieler muss Engpässe vorhersehen
* Hybrid: Spieler greift globale Märkte an → andere Spieler / KI reagieren

---

💡 **Takeaway:**

* Inventar + Bank → **Herzstück von Ruinborn**, weil sie **Ressourcen, Geld, Investitionen, PvP/PvE Interaktion** zusammenhalten
* Macht Spieleraktionen **greifbar und strategisch**
* Visualisierung: **Panels + 3D Map + Timeline** → Spieler sieht alles auf einen Blick
