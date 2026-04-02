import { useMemo } from "react";

/** Scatter low-poly trees across the map, avoiding trading post positions */
export default function Trees({ avoidPositions }: { avoidPositions: { x: number; z: number }[] }) {
  const trees = useMemo(() => {
    const result: { x: number; z: number; scale: number; color: string }[] = [];
    for (let i = 0; i < 100; i++) {
      const x = (Math.random() - 0.5) * 170;
      const z = (Math.random() - 0.5) * 170;
      const tooClose = avoidPositions.some((p) => Math.hypot(p.x - x, p.z - z) < 7);
      if (!tooClose) {
        const green = `#${(0x1a8030 + Math.floor(Math.random() * 0x224400)).toString(16).padStart(6, "0")}`;
        result.push({ x, z, scale: 0.6 + Math.random() * 0.5, color: green });
      }
    }
    return result;
  }, [avoidPositions]);

  return (
    <>
      {trees.map((t, i) => (
        <group key={i} position={[t.x, 0, t.z]}>
          {/* Trunk */}
          <mesh position={[0, 0.6, 0]} castShadow>
            <cylinderGeometry args={[0.1, 0.15, 1.2, 5]} />
            <meshStandardMaterial color="#5c3317" />
          </mesh>
          {/* Canopy */}
          <mesh position={[0, 1.5 * t.scale, 0]} castShadow>
            <sphereGeometry args={[0.7 * t.scale, 6, 5]} />
            <meshStandardMaterial color={t.color} flatShading />
          </mesh>
        </group>
      ))}
    </>
  );
}
