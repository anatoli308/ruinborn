import { Canvas } from "@react-three/fiber";
import { Stats } from "@react-three/drei";
import type { RefObject } from "react";
import Terrain from "./world/Terrain";
import Player from "./world/Player";
import FollowCamera from "./world/FollowCamera";
import { CAMERA_OFFSET } from "./world/cameraConfig";

interface GameWorldProps {
  /** Anchor element the FPS counter mounts into (positioned bottom-left in CSS). */
  fpsAnchorRef?: RefObject<HTMLDivElement | null>;
}

/**
 * Minimal Town-Build (Iteration A).
 *
 * Bewusst auf Town-Visuals reduziert — Enemies, Loot, Waypoints, Other Players,
 * Trading-Post-Meshes und Range-Indicator sind temporaer aus, bis das Town-Look
 * sitzt. Reaktiviert sobald wir Blood Moor angehen.
 *
 * Kamera-Offset zentral via cameraConfig (auch fuer Sprite-Tilt genutzt).
 */
export default function GameWorld({ fpsAnchorRef }: GameWorldProps) {
  return (
    <Canvas
      shadows
      camera={{
        position: [CAMERA_OFFSET.x, CAMERA_OFFSET.y, CAMERA_OFFSET.z],
        fov: 50,
        near: 0.1,
        far: 400,
      }}
      gl={{ antialias: true, toneMapping: 4, toneMappingExposure: 1.25 }}
      style={{ width: "100%", height: "100%" }}
    >
      {/* Sky / fog — town-warm, weit reichend fuer "endlosen" Boden. */}
      <color attach="background" args={["#1a1410"]} />
      <fog attach="fog" args={["#1a1410", 60, 180]} />

      {/* Lighting — etwas kraeftiger, sonst sind Sprites zu dunkel. */}
      <ambientLight intensity={0.85} color="#b8a890" />
      <directionalLight
        position={[40, 60, 25]}
        intensity={1.1}
        color="#ffe6c2"
        castShadow
        shadow-mapSize-width={2048}
        shadow-mapSize-height={2048}
        shadow-camera-left={-80}
        shadow-camera-right={80}
        shadow-camera-top={80}
        shadow-camera-bottom={-80}
      />
      <pointLight position={[0, 4, 0]} intensity={0.6} color="#ffaa55" distance={25} />

      <Terrain />
      <Player />
      <FollowCamera />

      <Stats parent={fpsAnchorRef as RefObject<HTMLElement>} />
    </Canvas>
  );
}
