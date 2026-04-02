import * as THREE from "three";
import { useMemo } from "react";
import { Line } from "@react-three/drei";
import { useGameStore } from "../../store/gameStore";

/** Road lines connecting trading posts */
export default function Roads() {
  const posts = useGameStore((s) => s.tradingPosts);

  const lines = useMemo(() => {
    const result: [THREE.Vector3, THREE.Vector3][] = [];
    const added = new Set<string>();

    for (const post of posts) {
      const sorted = [...posts]
        .filter((p) => p.id !== post.id)
        .sort((a, b) => Math.hypot(a.x - post.x, a.z - post.z) - Math.hypot(b.x - post.x, b.z - post.z));

      for (const target of sorted.slice(0, 2)) {
        const key = [post.id, target.id].sort().join("-");
        if (!added.has(key)) {
          added.add(key);
          result.push([
            new THREE.Vector3(post.x, 0.08, post.z),
            new THREE.Vector3(target.x, 0.08, target.z),
          ]);
        }
      }
    }
    return result;
  }, [posts]);

  return (
    <>
      {lines.map(([a, b], i) => (
        <Line
          key={i}
          points={[a, b]}
          color="#8b7355"
          lineWidth={1}
        />
      ))}
    </>
  );
}
