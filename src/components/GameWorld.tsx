import { Canvas } from "@react-three/fiber";
import { Stats } from "@react-three/drei";
import type { RefObject } from "react";
import Terrain from "./world/Terrain";
import TradingPostMesh from "./world/TradingPostMesh";
import Player from "./world/Player";
import OtherPlayers from "./world/OtherPlayers";
import FollowCamera from "./world/FollowCamera";
import Enemies from "./world/Enemies";
import LootDrops from "./world/LootDrops";
import Waypoints from "./world/Waypoints";
import RangeIndicator from "./world/RangeIndicator";
import { useGameStore } from "../store/gameStore";

interface GameWorldProps {
  /** Anchor element the FPS counter mounts into (positioned bottom-left in CSS). */
  fpsAnchorRef?: RefObject<HTMLDivElement | null>;
}

/** The full 3D game world rendered via R3F */
export default function GameWorld({ fpsAnchorRef }: GameWorldProps) {
  const playerMarkets = useGameStore((s) => s.playerMarkets);

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

      {playerMarkets.map((market) => (
        <TradingPostMesh key={market.id} market={market} />
      ))}

      {/* Enemies + Loot */}
      <Enemies />
      <LootDrops />

      {/* Waypoint stones — clickable in-world to open travel menu */}
      <Waypoints />

      {/* Player */}
      <Player />
      <OtherPlayers />
      <RangeIndicator />
      <FollowCamera />

      {/* FPS Counter (anchored bottom-left via parent ref + CSS) */}
      <Stats parent={fpsAnchorRef as RefObject<HTMLElement>} />
    </Canvas>
  );
}
