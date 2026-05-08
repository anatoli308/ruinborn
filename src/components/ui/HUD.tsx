import { useGameStore } from "../../store/gameStore";

/** Format elapsed seconds into "Day X HH:MM:SS" */
function formatElapsedTime(totalSecs: number): string {
  const totalSeconds = Math.floor(totalSecs);
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `Day ${days + 1} ${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
}

/** Top bar showing gold, day, reputation, controls */
export default function HUD() {
  const elapsedSecs = useGameStore((s) => s.elapsedSecs);
  const gold = useGameStore((s) => s.gold);
  const reputation = useGameStore((s) => s.reputation);
  const notification = useGameStore((s) => s.notification);
  const nearestMarketId = useGameStore((s) => s.nearestMarketId);
  const ownedMarketId = useGameStore((s) => s.ownedMarketId);
  const showTradePanel = useGameStore((s) => s.showTradePanel);
  const playerMarkets = useGameStore((s) => s.playerMarkets);
  const activeMissions = useGameStore((s) => s.activeMissions);
  const connected = useGameStore((s) => s.connected);
  const otherPlayers = useGameStore((s) => s.otherPlayers);
  const level = useGameStore((s) => s.level);
  const xp = useGameStore((s) => s.xp);
  const xpToNext = useGameStore((s) => s.xpToNext);
  const isDead = useGameStore((s) => s.isDead);
  const respawnIn = useGameStore((s) => s.respawnIn);
  const zone = useGameStore((s) => s.zone);

  const nearestMarket = playerMarkets.find((m) => m.id === nearestMarketId);

  return (
    <>
      {/* Connection status */}
      {!connected && (
        <div className="hud-notification">🔄 Connecting to server...</div>
      )}

      {/* Top Bar */}
      <div className="hud-top">
        <div className="hud-title">⚔ TradeWars</div>
        <div className="hud-stats">
          <span className="stat">🛡 Lv {level} ({xp}/{xpToNext} XP)</span>
          <span className="stat gold">💰 {Math.floor(gold).toLocaleString("en-US")} Gold</span>
          <span className="stat">📅 {formatElapsedTime(elapsedSecs)}</span>
          <span className="stat">⭐ {reputation} Rep</span>
          <span className="stat">🗺 {zone}</span>
          <span className="stat">👥 {otherPlayers.length + 1} Online</span>
        </div>
        <div className="hud-controls">
          <kbd>WASD</kbd> Move
          <kbd>LMB/RMB</kbd> Skill
          <kbd>T</kbd> Trade
          <kbd>I</kbd> Inventar
          <kbd>C</kbd> Charakter
          <kbd>K</kbd> Skills
          <kbd>1-9</kbd> Action
          {!ownedMarketId && <><kbd>M</kbd> Open Market</>}
        </div>
      </div>

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
