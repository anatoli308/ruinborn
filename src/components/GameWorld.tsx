import { Canvas } from "@react-three/fiber";
import { useMemo } from "react";
import { Stats } from "@react-three/drei";
import Terrain from "./world/Terrain";
import Trees from "./world/Trees";
import Water from "./world/Water";
import Roads from "./world/Roads";
import TradingPostMesh from "./world/TradingPostMesh";
import Player from "./world/Player";
import OtherPlayers from "./world/OtherPlayers";
import FollowCamera from "./world/FollowCamera";
import { useGameStore } from "../store/gameStore";

/** The full 3D game world rendered via R3F */
export default function GameWorld() {
  const tradingPosts = useGameStore((s) => s.tradingPosts);

  const postPositions = useMemo(
    () => tradingPosts.map((p) => ({ x: p.x, z: p.z })),
    [tradingPosts]
  );

  return (
    <Canvas
      shadows
      camera={{ position: [0, 18, 14], fov: 55, near: 0.1, far: 200 }}
      gl={{ antialias: true, toneMapping: 4, toneMappingExposure: 1.2 }}
      style={{ width: "100%", height: "100%" }}
    >
      {/* Sky / fog */}
      <color attach="background" args={["#1a1a2e"]} />
      <fog attach="fog" args={["#1a1a2e", 60, 130]} />

      {/* Lighting */}
      <ambientLight intensity={0.5} color="#8899aa" />
      <directionalLight
        position={[50, 80, 30]}
        intensity={1.0}
        color="#ffeedd"
        castShadow
        shadow-mapSize-width={2048}
        shadow-mapSize-height={2048}
        shadow-camera-left={-80}
        shadow-camera-right={80}
        shadow-camera-top={80}
        shadow-camera-bottom={-80}
      />
      <pointLight position={[0, 5, 0]} intensity={0.3} color="#ffd700" />

      {/* World */}
      <Terrain />
      <Water />
      <Roads />
      <Trees avoidPositions={postPositions} />

      {tradingPosts.map((post) => (
        <TradingPostMesh key={post.id} post={post} />
      ))}

      {/* Player */}
      <Player />
      <OtherPlayers />
      <FollowCamera />

      {/* FPS Counter (top-left) */}
      <Stats />
    </Canvas>
  );
}
