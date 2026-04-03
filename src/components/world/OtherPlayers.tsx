import { useGameStore } from "../../store/gameStore";

/** Renders other players in the world as simple meshes */
export default function OtherPlayers() {
  const otherPlayers = useGameStore((s) => s.otherPlayers);

  return (
    <>
      {otherPlayers.map((p) => (
        <group key={p.id} position={[p.x, 0, p.z]}>
          {/* Body */}
          <mesh position={[0, 0.8, 0]} castShadow>
            <capsuleGeometry args={[0.3, 0.6, 4, 8]} />
            <meshStandardMaterial color="#4488ff" roughness={0.4} metalness={0.3} />
          </mesh>
          {/* Head */}
          <mesh position={[0, 1.4, 0]} castShadow>
            <sphereGeometry args={[0.22, 8, 8]} />
            <meshStandardMaterial color="#aaccff" />
          </mesh>
          {/* Name tag */}
          {/* TODO: use Drei <Text> or <Html> for name labels */}
        </group>
      ))}
    </>
  );
}
