import { useGameStore } from "../../store/gameStore";

/** Bottom-right bag strip (5 bag slots — slot 0 fixed). */
export default function BagBar() {
  const bags = useGameStore((s) => s.bags);
  const inventoryOpen = useGameStore((s) => s.inventoryOpen);
  const toggleInventory = useGameStore((s) => s.toggleInventory);

  return (
    <div className="bag-bar">
      {bags.bags.map((bag, i) => (
        <button
          key={i}
          type="button"
          className={`bag-bar__slot${bag ? " bag-bar__slot--filled" : ""}${
            inventoryOpen ? " bag-bar__slot--open" : ""
          }`}
          title={bag ? bag.name : "Empty bag slot"}
          onClick={() => toggleInventory()}
        >
          {bag ? "🎒" : "·"}
        </button>
      ))}
    </div>
  );
}
