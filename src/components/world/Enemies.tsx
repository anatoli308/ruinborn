import { useGameStore } from "../../store/gameStore";
import type { Enemy } from "../../types";

const KIND_COLOR: Record<string, string> = {
  zombie: "#6b8e23",
  skeleton: "#cccccc",
  fallen_one: "#a0522d",
};

function EnemyMesh({ enemy }: { enemy: Enemy }) {
  const sendAttack = useGameStore((s) => s.sendAttack);

  if (enemy.state === "dead") return null;

  const color = KIND_COLOR[enemy.kind] ?? "#888";
  const hpPct = Math.max(0, enemy.hp / enemy.maxHp);

  return (
    <group position={[enemy.x, 0, enemy.z]}>
      <mesh
        position={[0, 0.6, 0]}
        castShadow
        onPointerDown={(e) => {
          e.stopPropagation();
          const button = e.button === 2 ? 1 : 0;
          // Server resolves: if mouse_left/right is a Skill, it casts; else basic attack.
          void sendAttack(enemy.id, button);
        }}
        onContextMenu={(e) => e.nativeEvent.preventDefault()}
      >
        <capsuleGeometry args={[0.4, 0.8, 4, 8]} />
        <meshStandardMaterial color={color} />
      </mesh>

      {/* HP bar */}
      <mesh position={[0, 1.6, 0]}>
        <planeGeometry args={[1, 0.1]} />
        <meshBasicMaterial color="#222" />
      </mesh>
      <mesh position={[-(1 - hpPct) / 2, 1.6, 0.001]}>
        <planeGeometry args={[hpPct, 0.1]} />
        <meshBasicMaterial color="#e74c3c" />
      </mesh>
    </group>
  );
}

export default function Enemies() {
  const enemies = useGameStore((s) => s.enemies);
  return (
    <>
      {enemies.map((e) => (
        <EnemyMesh key={e.id} enemy={e} />
      ))}
    </>
  );
}
