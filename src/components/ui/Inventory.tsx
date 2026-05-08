import { useGameStore } from "../../store/gameStore";

/** Mini inventory display in bottom-left */
export default function Inventory() {
  const inventory = useGameStore((s) => s.inventory);
  const commodities = useGameStore((s) => s.commodities);

  const items = Object.entries(inventory).filter(([, qty]) => qty > 0);
  if (items.length === 0) return null;

  return (
    <div className="hud-inventory">
      <div className="inv-title">📦 Inventory</div>
      {items.map(([id, qty]) => {
        const c = commodities.find((c) => c.id === id);
        if (!c) return null;
        return (
          <div key={id} className="inv-item">
            {c.icon} {c.name}: <strong>{qty}</strong>
          </div>
        );
      })}
    </div>
  );
}
