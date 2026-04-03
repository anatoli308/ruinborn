import { useGameStore } from "../../store/gameStore";

/** Top bar showing gold, day, reputation, controls */
export default function HUD() {
  const tick = useGameStore((s) => s.tick);
  const gold = useGameStore((s) => s.gold);
  const reputation = useGameStore((s) => s.reputation);
  const notification = useGameStore((s) => s.notification);
  const nearestPostId = useGameStore((s) => s.nearestPostId);
  const showTradePanel = useGameStore((s) => s.showTradePanel);
  const tradingPosts = useGameStore((s) => s.tradingPosts);
  const activeEvents = useGameStore((s) => s.activeEvents);
  const connected = useGameStore((s) => s.connected);
  const otherPlayers = useGameStore((s) => s.otherPlayers);

  const nearestPost = tradingPosts.find((p) => p.id === nearestPostId);

  return (
    <>
      {/* Connection status */}
      {!connected && (
        <div className="hud-notification">🔄 Verbinde mit Server...</div>
      )}

      {/* Top Bar */}
      <div className="hud-top">
        <div className="hud-title">⚔ TradeWars</div>
        <div className="hud-stats">
          <span className="stat gold">💰 {Math.floor(gold).toLocaleString("de-DE")} Gold</span>
          <span className="stat">📅 Tag {tick}</span>
          <span className="stat">⭐ {reputation} Ruf</span>
          <span className="stat">👥 {otherPlayers.length + 1} Online</span>
        </div>
        <div className="hud-controls">
          <kbd>WASD</kbd> Bewegen
          <kbd>E</kbd> Handeln
        </div>
      </div>

      {/* Interaction prompt */}
      {nearestPost && !showTradePanel && (
        <div className="hud-prompt">
          📍 <strong>{nearestPost.name}</strong> — Drücke <kbd>E</kbd> zum Handeln
        </div>
      )}

      {/* Notification */}
      {notification && (
        <div className="hud-notification">{notification}</div>
      )}

      {/* Active events */}
      {activeEvents.length > 0 && (
        <div className="hud-events">
          {activeEvents.map((e) => (
            <div key={e.id} className="event-item">
              ⚡ <strong>{e.name}</strong>: {e.description} ({e.remainingTicks} Tage)
            </div>
          ))}
        </div>
      )}
    </>
  );
}
