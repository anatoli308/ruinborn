import { useState } from "react";
import { useGameStore } from "../../store/gameStore";

/** Full-screen trade panel overlay when at a trading post */
export default function TradePanel() {
  const showTradePanel = useGameStore((s) => s.showTradePanel);
  const nearestPostId = useGameStore((s) => s.nearestPostId);
  const tradingPosts = useGameStore((s) => s.tradingPosts);
  const commodities = useGameStore((s) => s.commodities);
  const gold = useGameStore((s) => s.gold);
  const inventory = useGameStore((s) => s.inventory);
  const sendTrade = useGameStore((s) => s.sendTrade);
  const sendCloseTradePanel = useGameStore((s) => s.sendCloseTradePanel);

  const [qty, setQty] = useState(1);

  if (!showTradePanel || !nearestPostId) return null;
  const post = tradingPosts.find((p) => p.id === nearestPostId);
  if (!post) return null;

  const doTrade = async (commodityId: string, type: "buy" | "sell") => {
    await sendTrade(commodityId, type, qty);
  };

  return (
    <div className="trade-overlay">
      <div className="trade-panel">
        {/* Header */}
        <div className="tp-header">
          <div>
            <h2>{post.name}</h2>
            <span className="tp-level">Stufe {post.level}</span>
          </div>
          <div className="tp-gold">💰 {Math.floor(gold).toLocaleString("de-DE")} Gold</div>
          <button className="tp-close" onClick={sendCloseTradePanel}>✕</button>
        </div>

        {/* Quantity selector */}
        <div className="tp-qty-row">
          <span>Menge:</span>
          <button onClick={() => setQty(Math.max(1, qty - 1))}>−</button>
          <span className="tp-qty-val">{qty}</span>
          <button onClick={() => setQty(qty + 1)}>+</button>
          <button onClick={() => setQty(10)}>10</button>
          <button onClick={() => setQty(50)}>50</button>
        </div>

        {/* Commodity list */}
        <div className="tp-commodities">
          <div className="tp-row tp-row-header">
            <span>Ware</span>
            <span>Preis</span>
            <span>Trend</span>
            <span>Besitz</span>
            <span>Aktion</span>
          </div>
          {commodities.map((c) => {
            const isSpecialty = post.specialties.includes(c.id);
            const owned = inventory[c.id] || 0;
            const prev = c.priceHistory.length >= 2 ? c.priceHistory[c.priceHistory.length - 2] : c.price;
            const change = ((c.price - prev) / prev) * 100;
            const total = c.price * qty;
            const canBuy = gold >= total;
            const canSell = owned >= qty;

            return (
              <div key={c.id} className={`tp-row ${isSpecialty ? "tp-specialty" : ""}`}>
                <span className="tp-name">
                  {c.icon} {c.name}
                  {isSpecialty && <span className="tp-star"> ★</span>}
                </span>
                <span className="tp-price">
                  {c.price.toFixed(1)} G
                  <span className="tp-total">({total.toFixed(0)} G)</span>
                </span>
                <span className={`tp-change ${change >= 0 ? "up" : "down"}`}>
                  {change >= 0 ? "▲" : "▼"} {Math.abs(change).toFixed(1)}%
                </span>
                <span className="tp-owned">{owned}x</span>
                <span className="tp-actions">
                  <button
                    className="btn-buy"
                    disabled={!canBuy}
                    onClick={() => doTrade(c.id, "buy")}
                  >
                    Kaufen
                  </button>
                  <button
                    className="btn-sell"
                    disabled={!canSell}
                    onClick={() => doTrade(c.id, "sell")}
                  >
                    Verkaufen
                  </button>
                </span>
              </div>
            );
          })}
        </div>

        <div className="tp-hint">★ = Spezialität — Drücke <kbd>E</kbd> oder <kbd>Esc</kbd> zum Schließen</div>
      </div>
    </div>
  );
}
