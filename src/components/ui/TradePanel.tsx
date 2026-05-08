import { useState } from "react";
import { useGameStore } from "../../store/gameStore";

/** Order-book trade panel — shown when player is at a market */
export default function TradePanel() {
  const showTradePanel = useGameStore((s) => s.showTradePanel);
  const nearestMarketId = useGameStore((s) => s.nearestMarketId);
  const ownedMarketId = useGameStore((s) => s.ownedMarketId);
  const playerMarkets = useGameStore((s) => s.playerMarkets);
  const commodities = useGameStore((s) => s.commodities);
  const gold = useGameStore((s) => s.gold);
  const inventory = useGameStore((s) => s.inventory);
  const sendFillOrder = useGameStore((s) => s.sendFillOrder);
  const sendPostOrder = useGameStore((s) => s.sendPostOrder);
  const sendCancelOrder = useGameStore((s) => s.sendCancelOrder);
  const sendCloseTradePanel = useGameStore((s) => s.sendCloseTradePanel);

  const [fillQty, setFillQty] = useState(1);
  const [newCommodity, setNewCommodity] = useState("");
  const [newType, setNewType] = useState<"buy" | "sell">("sell");
  const [newQty, setNewQty] = useState(1);
  const [newPrice, setNewPrice] = useState(10);

  if (!showTradePanel || !nearestMarketId) return null;
  const market = playerMarkets.find((m) => m.id === nearestMarketId);
  if (!market) return null;

  const isOwnMarket = market.id === ownedMarketId;
  const sellOrders = market.orders.filter((o) => o.orderType === "sell");
  const buyOrders = market.orders.filter((o) => o.orderType === "buy");

  const getCommodity = (id: string) => commodities.find((c) => c.id === id);

  return (
    <div className="trade-overlay">
      <div className="trade-panel">
        {/* Header */}
        <div className="tp-header">
          <div>
            <h2>🏪 {market.name}</h2>
            <span className="tp-level">by {market.ownerName}{isOwnMarket ? " (Your Market)" : ""}</span>
          </div>
          <div className="tp-gold">💰 {Math.floor(gold).toLocaleString("en-US")} Gold</div>
          <button className="tp-close" onClick={sendCloseTradePanel}>✕</button>
        </div>

        {/* Fill quantity selector (for other markets) */}
        {!isOwnMarket && (
          <div className="tp-qty-row">
            <span>Qty:</span>
            <button onClick={() => setFillQty(Math.max(1, fillQty - 1))}>−</button>
            <span className="tp-qty-val">{fillQty}</span>
            <button onClick={() => setFillQty(fillQty + 1)}>+</button>
            <button onClick={() => setFillQty(5)}>5</button>
            <button onClick={() => setFillQty(10)}>10</button>
          </div>
        )}

        {/* Sell orders — visitor can buy from these */}
        <div className="tp-commodities">
          <h3 style={{ margin: "0.5rem 0", color: "#ff6b6b" }}>📤 Sell Orders</h3>
          {sellOrders.length === 0 && (
            <div className="tp-row" style={{ opacity: 0.5 }}>No sell orders</div>
          )}
          {sellOrders.map((o) => {
            const c = getCommodity(o.commodityId);
            return (
              <div key={o.id} className="tp-row">
                <span className="tp-name">{c?.icon} {c?.name}</span>
                <span className="tp-price">{o.pricePerUnit.toFixed(1)} G/ea</span>
                <span className="tp-owned">{o.remaining}x</span>
                <span className="tp-actions">
                  {isOwnMarket ? (
                    <button className="btn-sell" onClick={() => sendCancelOrder(o.id)}>Cancel</button>
                  ) : (
                    <button
                      className="btn-buy"
                      disabled={gold < o.pricePerUnit * fillQty}
                      onClick={() => sendFillOrder(market.id, o.id, fillQty)}
                    >
                      Buy
                    </button>
                  )}
                </span>
              </div>
            );
          })}
        </div>

        {/* Buy orders — visitor can sell to these */}
        <div className="tp-commodities">
          <h3 style={{ margin: "0.5rem 0", color: "#51cf66" }}>📥 Buy Orders</h3>
          {buyOrders.length === 0 && (
            <div className="tp-row" style={{ opacity: 0.5 }}>No buy orders</div>
          )}
          {buyOrders.map((o) => {
            const c = getCommodity(o.commodityId);
            const owned = inventory[o.commodityId] || 0;
            return (
              <div key={o.id} className="tp-row">
                <span className="tp-name">{c?.icon} {c?.name}</span>
                <span className="tp-price">{o.pricePerUnit.toFixed(1)} G/ea</span>
                <span className="tp-owned">{o.remaining}x</span>
                <span className="tp-actions">
                  {isOwnMarket ? (
                    <button className="btn-sell" onClick={() => sendCancelOrder(o.id)}>Cancel</button>
                  ) : (
                    <button
                      className="btn-sell"
                      disabled={owned < fillQty}
                      onClick={() => sendFillOrder(market.id, o.id, fillQty)}
                    >
                      Sell
                    </button>
                  )}
                </span>
              </div>
            );
          })}
        </div>

        {/* Post new order (only for own market) */}
        {isOwnMarket && (
          <div className="tp-commodities" style={{ marginTop: "1rem" }}>
            <h3 style={{ margin: "0.5rem 0", color: "#ffd700" }}>📋 Create New Order</h3>
            <div className="tp-qty-row" style={{ flexWrap: "wrap", gap: "0.5rem" }}>
              <select
                value={newCommodity}
                onChange={(e) => setNewCommodity(e.target.value)}
                style={{ background: "#1a1e2a", color: "white", border: "1px solid #333", padding: "4px" }}
              >
                <option value="">— Commodity —</option>
                {commodities.map((c) => (
                  <option key={c.id} value={c.id}>{c.icon} {c.name}</option>
                ))}
              </select>
              <select
                value={newType}
                onChange={(e) => setNewType(e.target.value as "buy" | "sell")}
                style={{ background: "#1a1e2a", color: "white", border: "1px solid #333", padding: "4px" }}
              >
                <option value="sell">Sell</option>
                <option value="buy">Buy</option>
              </select>
              <span>Qty:</span>
              <button onClick={() => setNewQty(Math.max(1, newQty - 1))}>−</button>
              <span className="tp-qty-val">{newQty}</span>
              <button onClick={() => setNewQty(newQty + 1)}>+</button>
              <span>Price:</span>
              <input
                type="number"
                min={1}
                step={1}
                value={newPrice}
                onChange={(e) => setNewPrice(Math.max(1, Number(e.target.value)))}
                style={{ width: "60px", background: "#1a1e2a", color: "white", border: "1px solid #333", padding: "4px" }}
              />
              <button
                className="btn-buy"
                disabled={!newCommodity}
                onClick={() => {
                  if (newCommodity) sendPostOrder(newCommodity, newType, newQty, newPrice);
                }}
              >
                Place Order
              </button>
            </div>
          </div>
        )}

        <div className="tp-hint">Press <kbd>E</kbd> or <kbd>Esc</kbd> to close</div>
      </div>
    </div>
  );
}
