import { useFrame } from "@react-three/fiber";
import { useMemo, useRef } from "react";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";
import type { Enemy, EnemyKind } from "../../types";
import FlareSprite, { angleToFlareDirection } from "./FlareSprite";
import { getNpcFlareLayer, NPC_FLARE_ATLASES } from "../../assets/npc/flare";

/** Map server EnemyKind → NPC atlas name in `assets/npc/atlases/`. */
const KIND_TO_ATLAS: Record<EnemyKind, string> = {
  zombie: "zombie",
  skeleton: "skeleton",
  fallen_one: "goblin",
};

/** Smoothing rate for enemy position lerp. Higher = snappier. */
const POSITION_SMOOTH_RATE = 14;
/** Distance above which we snap (teleport / respawn). */
const POSITION_SNAP_DISTANCE = 8;

/** Pick an animation name supported by the atlas — fall back to stance. */
function pickAnimation(atlasName: string, desired: string): string {
  const atlas = NPC_FLARE_ATLASES[atlasName];
  if (!atlas) return desired;
  if (atlas.animations[desired]) return desired;
  if (atlas.animations.stance) return "stance";
  // Last resort: first available animation.
  const first = Object.keys(atlas.animations)[0];
  return first ?? desired;
}

function EnemyMesh({ enemy }: { enemy: Enemy }) {
  const sendAttack = useGameStore((s) => s.sendAttack);
  const setTargetEnemy = useGameStore((s) => s.setTargetEnemy);
  const targetEnemyId = useGameStore((s) => s.targetEnemyId);

  const groupRef = useRef<THREE.Group>(null);
  const ringRef = useRef<THREE.Mesh>(null);
  const animationRef = useRef("stance");
  const directionRef = useRef(0);

  /** Locally smoothed position. Initialized lazily on first frame. */
  const displayed = useRef<{ x: number; z: number; init: boolean }>({
    x: enemy.x,
    z: enemy.z,
    init: false,
  });
  /** Last frame's position so we can derive a velocity for facing. */
  const lastDisplayed = useRef<{ x: number; z: number }>({ x: enemy.x, z: enemy.z });

  const isTarget = targetEnemyId === enemy.id;
  const atlasName = KIND_TO_ATLAS[enemy.kind] ?? "zombie";

  // Layers list is just the single body atlas — NPCs are flat sprites.
  const layers = useMemo(() => [atlasName], [atlasName]);

  useFrame((_, delta) => {
    if (!groupRef.current) return;

    if (!displayed.current.init) {
      displayed.current.x = enemy.x;
      displayed.current.z = enemy.z;
      lastDisplayed.current.x = enemy.x;
      lastDisplayed.current.z = enemy.z;
      displayed.current.init = true;
    }

    const dx = enemy.x - displayed.current.x;
    const dz = enemy.z - displayed.current.z;
    const dist = Math.hypot(dx, dz);

    if (dist > POSITION_SNAP_DISTANCE) {
      displayed.current.x = enemy.x;
      displayed.current.z = enemy.z;
    } else {
      // Frame-rate independent exponential smoothing.
      const t = 1 - Math.exp(-POSITION_SMOOTH_RATE * delta);
      displayed.current.x += dx * t;
      displayed.current.z += dz * t;
    }

    // Derive animation + facing from displayed velocity.
    const vx = displayed.current.x - lastDisplayed.current.x;
    const vz = displayed.current.z - lastDisplayed.current.z;
    const speed = Math.hypot(vx, vz);
    const moving = speed > 0.001;
    if (moving) {
      const facing = Math.atan2(vx, -vz);
      directionRef.current = angleToFlareDirection(facing);
    }
    animationRef.current = pickAnimation(atlasName, moving ? "run" : "stance");

    lastDisplayed.current.x = displayed.current.x;
    lastDisplayed.current.z = displayed.current.z;

    groupRef.current.position.set(displayed.current.x, 0.2, displayed.current.z);

    if (ringRef.current && isTarget) {
      const s = 1 + Math.sin(performance.now() * 0.005) * 0.08;
      ringRef.current.scale.set(s, s, s);
    }
  });

  if (enemy.state === "dead") return null;

  const hpPct = Math.max(0, enemy.hp / enemy.maxHp);
  const hasAtlas = getNpcFlareLayer(atlasName) !== null;

  return (
    <group ref={groupRef}>
      {/* Click hitbox — invisible capsule so the sprite doesn't have to be clickable. */}
      <mesh
        position={[0, 0.6, 0]}
        visible={false}
        onPointerDown={(e) => {
          e.stopPropagation();
          const button = e.button === 2 ? 1 : 0;
          setTargetEnemy(enemy.id);
          void sendAttack(enemy.id, button);
        }}
        onContextMenu={(e) => e.nativeEvent.preventDefault()}
      >
        <capsuleGeometry args={[0.4, 0.8, 4, 8]} />
        <meshBasicMaterial transparent opacity={0} depthWrite={false} />
      </mesh>

      {/* Sprite (FLARE NPC atlas) — falls back to a colored capsule if missing. */}
      {hasAtlas ? (
        <FlareSprite
          layers={layers}
          animationRef={animationRef}
          directionRef={directionRef}
          resolveLayer={getNpcFlareLayer}
        />
      ) : (
        <mesh position={[0, 0.6, 0]} castShadow>
          <capsuleGeometry args={[0.4, 0.8, 4, 8]} />
          <meshStandardMaterial color="#888" />
        </mesh>
      )}

      {/* Selection ring (D2/WoW style) — only visible for the current target. */}
      {isTarget && (
        <mesh ref={ringRef} position={[0, 0.05, 0]} rotation={[-Math.PI / 2, 0, 0]}>
          <ringGeometry args={[0.7, 0.85, 32]} />
          <meshBasicMaterial color="#facc15" transparent opacity={0.9} side={THREE.DoubleSide} />
        </mesh>
      )}

      {/* HP bar above sprite */}
      <mesh position={[0, 1.6, 0]}>
        <planeGeometry args={[1, 0.1]} />
        <meshBasicMaterial color="#222" />
      </mesh>
      <mesh position={[-(1 - hpPct) / 2, 1.6, 0.001]}>
        <planeGeometry args={[hpPct, 0.1]} />
        <meshBasicMaterial color="#e74c3c" />
      </mesh>
    </group>
  );
}

export default function Enemies() {
  const enemies = useGameStore((s) => s.enemies);
  return (
    <>
      {enemies.map((e) => (
        <EnemyMesh key={e.id} enemy={e} />
      ))}
    </>
  );
}
