import { useEffect, useMemo, useRef } from "react";
import { useFrame, type ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";

/** Waypoint stones in world coordinates — must match `crates/ruinborn-game/src/world.rs`. */
const WAYPOINTS: Array<{ id: string; x: number; z: number }> = [
  { id: "town", x: 0, z: 0 },
  { id: "wilderness", x: 0, z: 60 },
  { id: "burial_grounds", x: 90, z: 75 },
];

/** 4-frame animation loop, ~150ms per frame. */
const FRAME_FILES = import.meta.glob<string>(
  "../../assets/misc/gowaypoint_*.png",
  { eager: true, import: "default", query: "?url" },
);
const FRAME_URLS: string[] = Object.keys(FRAME_FILES)
  .sort()
  .map((k) => FRAME_FILES[k]);

const FRAME_DURATION_MS = 150;
/** Diameter of the floor decal in world units. */
const DECAL_SIZE = 3.0;
/** Lift slightly above the ground plane to avoid z-fighting with terrain. */
const DECAL_Y = 0.03;

function createPixelTexture(url: string): THREE.Texture {
  const tex = new THREE.Texture();
  tex.magFilter = THREE.NearestFilter;
  tex.minFilter = THREE.NearestFilter;
  tex.generateMipmaps = false;
  tex.colorSpace = THREE.SRGBColorSpace;
  const img = new Image();
  img.onload = () => {
    tex.image = img;
    tex.needsUpdate = true;
  };
  img.src = url;
  return tex;
}

interface WaypointProps {
  position: [number, number, number];
}

/**
 * Flat ground decal rendered in the XZ plane (Diablo 2-style waypoint rune).
 * No camera billboarding — the disc lies flat on the floor and reads naturally
 * from the isometric view.
 */
function Waypoint({ position }: WaypointProps) {
  const matRef = useRef<THREE.MeshBasicMaterial>(null!);
  const elapsedRef = useRef(Math.random() * FRAME_DURATION_MS * FRAME_URLS.length);
  const setWaypointMenuOpen = useGameStore((s) => s.setWaypointMenuOpen);

  // One Texture per frame so we can swap `material.map` cheaply.
  const textures = useMemo(() => FRAME_URLS.map(createPixelTexture), []);

  useEffect(() => {
    return () => {
      for (const t of textures) t.dispose();
    };
  }, [textures]);

  useFrame((_, delta) => {
    elapsedRef.current += delta * 1000;
    const totalFrames = textures.length;
    if (totalFrames === 0) return;
    const idx =
      Math.floor(elapsedRef.current / FRAME_DURATION_MS) % totalFrames;
    const tex = textures[idx];
    const mat = matRef.current;
    if (mat && mat.map !== tex) {
      mat.map = tex;
      mat.needsUpdate = true;
    }
  });

  const handleClick = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    setWaypointMenuOpen(true);
  };

  const handleOver = (e: ThreeEvent<PointerEvent>) => {
    e.stopPropagation();
    document.body.style.cursor = "pointer";
  };

  const handleOut = () => {
    document.body.style.cursor = "";
  };

  return (
    <mesh
      position={position}
      rotation={[-Math.PI / 2, 0, 0]}
      onClick={handleClick}
      onPointerOver={handleOver}
      onPointerOut={handleOut}
    >
      <planeGeometry args={[DECAL_SIZE, DECAL_SIZE]} />
      <meshBasicMaterial
        ref={matRef}
        transparent
        alphaTest={0.5}
        depthWrite={false}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

/** All in-world waypoint stones — clicking one opens the travel menu. */
export default function Waypoints() {
  return (
    <>
      {WAYPOINTS.map((wp) => (
        <Waypoint key={wp.id} position={[wp.x, DECAL_Y, wp.z]} />
      ))}
    </>
  );
}
