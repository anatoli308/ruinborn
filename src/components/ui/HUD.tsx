import { useGameStore } from "../../store/gameStore";

/** Floating HUD overlays — prompts, notifications, missions, death. */
export default function HUD() {
  const notification = useGameStore((s) => s.notification);
  const nearestMarketId = useGameStore((s) => s.nearestMarketId);
  const ownedMarketId = useGameStore((s) => s.ownedMarketId);
  const showTradePanel = useGameStore((s) => s.showTradePanel);
  const playerMarkets = useGameStore((s) => s.playerMarkets);
  const activeMissions = useGameStore((s) => s.activeMissions);
  const connected = useGameStore((s) => s.connected);
  const isDead = useGameStore((s) => s.isDead);
  const respawnIn = useGameStore((s) => s.respawnIn);

  const nearestMarket = playerMarkets.find((m) => m.id === nearestMarketId);
  // `ownedMarketId` is informational only here; the [M] hotkey lives in Player.tsx.
  void ownedMarketId;

  return (
    <>
      {/* Connection status */}
      {!connected && (
        <div className="hud-notification">🔄 Connecting to server...</div>
      )}

      {/* Market interaction prompt */}
      {nearestMarket && !showTradePanel && (
        <div className="hud-prompt">
          🏪 <strong>{nearestMarket.name}</strong> — Press <kbd>E</kbd> to trade
        </div>
      )}

      {/* Death overlay */}
      {isDead && (
        <div className="hud-notification" style={{ color: "#e74c3c", fontSize: "1.5em" }}>
          ☠ Du bist gestorben — Respawn in {respawnIn} Ticks
        </div>
      )}

      {/* Notification */}
      {notification && (
        <div className="hud-notification">{notification}</div>
      )}

      {/* Active missions */}
      {activeMissions.length > 0 && (
        <div className="hud-events">
          {activeMissions.map((m) => (
            <div key={m.id} className="event-item">
              📜 <strong>{m.title}</strong> ({m.progress}/{m.targetQuantity})
            </div>
          ))}
        </div>
      )}
    </>
  );
}
