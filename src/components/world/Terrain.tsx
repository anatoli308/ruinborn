import { useMemo } from "react";
import * as THREE from "three";

/** Low-poly terrain with subtle height variation */
export default function Terrain() {
  const geometry = useMemo(() => {
    const geo = new THREE.PlaneGeometry(200, 200, 50, 50);
    const pos = geo.attributes.position;
    for (let i = 0; i < pos.count; i++) {
      const x = pos.getX(i);
      const y = pos.getY(i);
      pos.setZ(i, Math.sin(x * 0.04) * Math.cos(y * 0.04) * 0.6 + Math.sin(x * 0.1 + y * 0.08) * 0.2);
    }
    geo.computeVertexNormals();
    return geo;
  }, []);

  return (
    <>
      <mesh rotation={[-Math.PI / 2, 0, 0]} receiveShadow geometry={geometry}>
        <meshStandardMaterial color="#2d5016" roughness={0.9} flatShading />
      </mesh>
      <gridHelper args={[200, 40, "#1a3a0a", "#1a3a0a"]} position={[0, 0.02, 0]} />
    </>
  );
}
