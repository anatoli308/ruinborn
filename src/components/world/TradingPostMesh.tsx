import { Text } from "@react-three/drei";
import type { PlayerMarket } from "../../types";

/** 3D player market building */
export default function TradingPostMesh({ market }: { market: PlayerMarket }) {
  const h = 2.5;

  return (
    <group position={[market.x, 0, market.z]}>
      {/* Platform */}
      <mesh position={[0, 0.15, 0]} castShadow receiveShadow>
        <cylinderGeometry args={[3, 3.5, 0.3, 8]} />
        <meshStandardMaterial color="#8b7355" />
      </mesh>

      {/* Building */}
      <mesh position={[0, h / 2 + 0.3, 0]} castShadow>
        <boxGeometry args={[2.5, h, 2]} />
        <meshStandardMaterial color="#daa520" roughness={0.7} />
      </mesh>

      {/* Roof */}
      <mesh position={[0, h + 0.8 + 0.3, 0]} rotation={[0, Math.PI / 4, 0]} castShadow>
        <coneGeometry args={[2, 1.5, 4]} />
        <meshStandardMaterial color="#b8860b" />
      </mesh>

      {/* Flag pole + flag */}
      <mesh position={[1.5, 2.5, 0]}>
        <cylinderGeometry args={[0.04, 0.04, 2, 5]} />
        <meshStandardMaterial color="#555" />
      </mesh>
      <mesh position={[1.9, 3.3, 0]}>
        <planeGeometry args={[0.7, 0.4]} />
        <meshStandardMaterial color="#ffd700" side={2} />
      </mesh>

      {/* Name label */}
      <Text
        position={[0, h + 2.5, 0]}
        fontSize={0.6}
        color="white"
        anchorX="center"
        anchorY="bottom"
        outlineWidth={0.04}
        outlineColor="black"
      >
        {market.name}
      </Text>
      <Text
        position={[0, h + 1.8, 0]}
        fontSize={0.35}
        color="#aaa"
        anchorX="center"
        anchorY="bottom"
      >
        von {market.ownerName}
      </Text>
    </group>
  );
}
