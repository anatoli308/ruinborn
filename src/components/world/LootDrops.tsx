import { useGameStore } from "../../store/gameStore";
import type { LootDrop, Rarity } from "../../types";

const RARITY_COLOR: Record<Rarity, string> = {
  Common: "#cccccc",
  Magic: "#4a90e2",
  Rare: "#f1c40f",
  Epic: "#b87333",
  Legendary: "#27ae60",
};

function LootMesh({ loot }: { loot: LootDrop }) {
  const sendPickupLoot = useGameStore((s) => s.sendPickupLoot);
  const color = RARITY_COLOR[loot.item.rarity] ?? "#cccccc";

  return (
    <mesh
      position={[loot.x, 0.3, loot.z]}
      onPointerDown={(e) => {
        e.stopPropagation();
        void sendPickupLoot(loot.id);
      }}
    >
      <octahedronGeometry args={[0.25, 0]} />
      <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.6} />
    </mesh>
  );
}

export default function LootDrops() {
  const lootDrops = useGameStore((s) => s.lootDrops);
  return (
    <>
      {lootDrops.map((l) => (
        <LootMesh key={l.id} loot={l} />
      ))}
    </>
  );
}
