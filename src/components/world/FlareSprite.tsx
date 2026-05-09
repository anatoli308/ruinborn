import { useEffect, useMemo, useRef } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import * as THREE from "three";

import {
  type FlareAnimation,
  type FlareAtlas,
  getFlareLayer,
  pickFrame,
} from "../../assets/player_male/flare";

/** Resolves a layer name to its atlas + bundled PNG URL. */
export type FlareLayerResolver = (
  name: string,
) => { atlas: FlareAtlas; imageUrl: string } | null;

const PIXELS_PER_UNIT_DEFAULT = 32;
/** Tiny Z gap between layers so depthWrite resolves them deterministically. */
const LAYER_Z_STEP = 0.001;

/**
 * Direction (0..7) selector. FLARE uses 8 frames per animation row.
 * The angle is the player's facing in radians where 0 = -Z (north),
 * π/2 = +X (east), produced by `atan2(dx, -dz)`.
 *
 * The shipped FLARE assets order their direction row as:
 *   dir 0 = West, dir 2 = North, dir 4 = East, dir 6 = South.
 * Compared to a naive north-zero CCW lookup that's a +2 octant offset.
 */
export function angleToFlareDirection(angle: number): number {
  const step = Math.PI / 4;
  const raw = Math.round(angle / step) + 2;
  return ((raw % 8) + 8) % 8;
}

/** Build a Three.Texture that loads asynchronously without suspending React. */
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

interface FlareLayerMeshProps {
  atlas: FlareAtlas;
  texture: THREE.Texture;
  animationRef: React.MutableRefObject<string>;
  directionRef: React.MutableRefObject<number>;
  pixelsPerUnit: number;
  zOffset: number;
}

function FlareLayerMesh({
  atlas,
  texture,
  animationRef,
  directionRef,
  pixelsPerUnit,
  zOffset,
}: FlareLayerMeshProps) {
  const meshRef = useRef<THREE.Mesh>(null!);
  const elapsedRef = useRef(0);
  const lastAnimRef = useRef("");

  useFrame((_, delta) => {
    const animName = animationRef.current;
    const anim: FlareAnimation | undefined = atlas.animations[animName];
    const mesh = meshRef.current;
    if (!anim || !mesh) return;

    // Reset playhead when the animation changes so play_once cues from frame 0.
    if (lastAnimRef.current !== animName) {
      elapsedRef.current = 0;
      lastAnimRef.current = animName;
    }
    elapsedRef.current += delta * 1000;

    const frame = pickFrame(anim, elapsedRef.current, directionRef.current);
    const img = texture.image as HTMLImageElement | undefined;
    const texW = img?.naturalWidth ?? 0;
    const texH = img?.naturalHeight ?? 0;
    if (!frame || texW === 0 || texH === 0) {
      mesh.visible = false;
      return;
    }
    mesh.visible = true;

    // UV slicing — Three's default flipY=true means UV (0,0) is bottom-left of
    // the (already-flipped) image, so the bottom of the frame in source pixels
    // is `1 - (y + h) / texH` along V.
    texture.offset.set(frame.x / texW, 1 - (frame.y + frame.h) / texH);
    texture.repeat.set(frame.w / texW, frame.h / texH);

    // Plane is unit-sized; we scale it to the frame's pixel dimensions and
    // shift its center so the character's feet (the FLARE pivot) sit at the
    // parent group's origin.
    const s = 1 / pixelsPerUnit;
    mesh.scale.set(frame.w * s, frame.h * s, 1);
    mesh.position.set(
      (frame.w / 2 - frame.ox) * s,
      (frame.oy - frame.h / 2) * s,
      zOffset,
    );
  });

  return (
    <mesh ref={meshRef}>
      <planeGeometry args={[1, 1]} />
      <meshBasicMaterial
        map={texture}
        transparent
        alphaTest={0.5}
        depthWrite
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

interface FlareSpriteProps {
  /** Atlas keys, in render order from back to front (e.g. legs → chest → head). */
  layers: string[];
  /**
   * Refs the parent updates each frame — sprites read them in their own
   * `useFrame` without triggering React re-renders.
   */
  animationRef: React.MutableRefObject<string>;
  directionRef: React.MutableRefObject<number>;
  /** World units per source pixel divisor (higher = smaller sprite). */
  pixelsPerUnit?: number;
  /** Disable camera-facing billboarding (debug). */
  noBillboard?: boolean;
  /**
   * Custom layer resolver — defaults to the player_male atlas table.
   * Pass `getNpcFlareLayer` (from `assets/npc/flare`) to render NPC sprites.
   */
  resolveLayer?: FlareLayerResolver;
}

/**
 * Renders a stack of FLARE animation atlases as camera-facing billboards.
 * All layers share the same pivot (feet anchor at the group's origin) so they
 * align automatically. Place this inside a parent group whose position is the
 * character's foot world-position.
 */
export default function FlareSprite({
  layers,
  animationRef,
  directionRef,
  pixelsPerUnit = PIXELS_PER_UNIT_DEFAULT,
  noBillboard = false,
  resolveLayer = getFlareLayer,
}: FlareSpriteProps) {
  const groupRef = useRef<THREE.Group>(null!);
  const { camera } = useThree();

  // Resolve each layer once — the resolver reads from an eager-loaded glob.
  const resolved = useMemo(() => {
    return layers
      .map((name) => {
        const layer = resolveLayer(name);
        if (!layer) {
          console.warn(`[FlareSprite] missing atlas/image: ${name}`);
          return null;
        }
        return { name, atlas: layer.atlas, imageUrl: layer.imageUrl };
      })
      .filter((x): x is { name: string; atlas: FlareAtlas; imageUrl: string } => x !== null);
  }, [layers, resolveLayer]);

  // Each layer needs its own Texture instance (independent offset/repeat).
  const textures = useMemo(
    () => resolved.map((r) => createPixelTexture(r.imageUrl)),
    [resolved],
  );

  // Dispose textures on unmount to avoid GPU leaks.
  useEffect(() => {
    return () => {
      for (const t of textures) t.dispose();
    };
  }, [textures]);

  useFrame(() => {
    if (noBillboard) return;
    const g = groupRef.current;
    if (!g) return;
    // Y-axis-locked billboard: face the camera horizontally only.
    g.lookAt(camera.position.x, g.position.y, camera.position.z);
  });

  return (
    <group ref={groupRef}>
      {resolved.map((r, i) => (
        <FlareLayerMesh
          key={r.name}
          atlas={r.atlas}
          texture={textures[i]}
          animationRef={animationRef}
          directionRef={directionRef}
          pixelsPerUnit={pixelsPerUnit}
          zOffset={i * LAYER_Z_STEP}
        />
      ))}
    </group>
  );
}
