# TradeWars — Networking & Performance

## Tick-System

TradeWars nutzt zwei getrennte Tick-Raten:

| Tick | Rate | Intervall | Zweck |
|------|------|-----------|-------|
| **Network Tick** | 20 Hz | 50 ms | Positionen, UI-State, Player-Updates |
| **Economy Tick** | 1 Hz | 1000 ms | Commodity-Preise, Events, Supply/Demand |

Die Economy tickt langsamer, weil Preisformeln, Volatility und Event-Dauern auf "1 Tick = 1 Spielsekunde" balanciert sind. Die Network-Rate ist unabhängig davon konfigurierbar.

```
server/main.rs:
  NETWORK_TICK_MS = 50          ← 20 Hz Broadcast
  ECONOMY_TICK_INTERVAL = 20    ← Economy alle 20 Network-Ticks (= 1s)
```

## Delta-Updates

Statt jedes Mal den vollständigen State zu senden, unterscheidet der Server zwei Message-Typen:

### `"state"` — Full Snapshot
- Enthält: Player, OtherPlayers, Commodities, TradingPosts, ActiveEvents
- Gesendet bei: **Join**, **Trade**, **Panel-Toggle**
- Zweck: Initialer State + nach Mutationen die den vollen Kontext brauchen

### `"delta"` — Delta Snapshot
- Enthält **immer**: Player, OtherPlayers (Positionen ändern sich ständig)
- Enthält **nur bei Economy-Tick**: Commodities, ActiveEvents
- Enthält **nie**: TradingPosts (ändern sich quasi nie)
- Gesendet bei: **Jedem 50ms Broadcast-Tick**

### Bandbreiten-Effekt

| Daten | Full Snapshot | Delta (ohne Economy) |
|-------|--------------|---------------------|
| Player + OtherPlayers | ~300 B | ~300 B |
| Commodities (8 Stück) | ~1500 B | 0 B |
| TradingPosts (5 Stück) | ~800 B | 0 B |
| ActiveEvents | ~200 B | 0 B |
| **Gesamt** | **~2800 B** | **~300 B** |

→ 19 von 20 Ticks senden nur ~300 B statt ~2800 B = **~90% weniger Traffic** im Durchschnitt.

## Client-side Prediction

Für ein responsives Spielgefühl berechnet der Client die Spielerposition **lokal sofort**, ohne auf den Server zu warten.

### So funktioniert es

```
Frame N:  Spieler drückt W
          → Predicted Position: z -= SPEED * delta   (sofort sichtbar)
          → sendMove(0, -dz) an Server

Frame N+1 bis N+X:  Prediction läuft weiter

Tick:     Server bestätigt Position (playerX, playerZ im Delta)
          → predictedPos lerpt Richtung Server-Position
          → RECONCILE_LERP = 0.15 (weicher Übergang)
```

### Reconciliation

```typescript
// Jeder Frame:
predictedPos.x += (serverX - predictedPos.x) * 0.15;
predictedPos.z += (serverZ - predictedPos.z) * 0.15;
```

- Bei **0 Latenz** (localhost): Predicted und Server-Position sind quasi identisch
- Bei **50ms Latenz**: ~1 Frame Versatz, unsichtbar
- Bei **200ms Latenz**: Leichte Korrektur sichtbar, aber kein Teleporting

### World Bounds

Die Client-Prediction spiegelt die Server-Logik:
```typescript
predictedPos.x = Math.max(-90, Math.min(90, predictedPos.x));
```
Das verhindert, dass der Spieler visuell über die Weltgrenze läuft und dann zurückgezogen wird.

## WebSocket Transport

### Verbindung
- Singleton WebSocket zu `ws://localhost:9000`
- Auto-Reconnect mit 2000ms Delay bei Disconnect
- Stale Sockets werden vor Neuverbindung geschlossen

### Message Format
- JSON-basiert (Text-Frames)
- Client → Server: `{ "cmd": "move", "dx": 0.5, "dz": -0.3 }`
- Server → Client: `{ "type": "delta", "snapshot": { ... } }`

### Trade-Promise Pattern
`sendTrade()` gibt ein Promise zurück, das vom asynchronen `"trade_result"` Message aufgelöst wird:
```typescript
sendTrade() → Promise
    ↓ WebSocket send
Server validiert
    ↓ WebSocket receive "trade_result"
Promise resolved
```
Timeout nach 5 Sekunden falls der Server nicht antwortet.

## Konfigurierbare Parameter

| Parameter | Datei | Aktuell | Wirkung |
|-----------|-------|---------|---------|
| `NETWORK_TICK_MS` | server/main.rs | 50 | Broadcast-Intervall (ms) |
| `ECONOMY_TICK_INTERVAL` | server/main.rs | 20 | Economy-Ticks pro Network-Tick |
| `RECONCILE_LERP` | Player.tsx | 0.15 | Prediction → Server Übergangsrate |
| `SPEED` | Player.tsx | 12 | Spieler-Geschwindigkeit (Einheiten/s) |
| `RECONNECT_DELAY_MS` | wsTransport.ts | 2000 | Reconnect-Wartezeit (ms) |

## Skalierungs-Roadmap

Aktuell nicht implementiert, aber relevante nächste Schritte:

1. **Binary Serialization** (MessagePack statt JSON) → ~50% weniger Bytes
2. **Interest Management** → Nur Spieler in der Nähe senden
3. **Horizontal Sharding** → Zonen auf verschiedene Prozesse
4. **Connection Pooling** → Mehrere Server hinter Load-Balancer
