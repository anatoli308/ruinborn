import { useMemo, useRef } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";
import { SKILL_CATALOG, targetingKind, type SkillTargetingKind } from "../../data/classes";

const Y_OFFSET = 0.05;
const RING_THICKNESS = 0.18;
const RING_COLOR = "#7ec8ff";
const AOE_COLOR = "#ffb066";
const SKILLSHOT_COLOR = "#ff6b6b";
const SKILLSHOT_WIDTH = 0.45;

/**
 * Active descriptor — what the world should currently visualize. Hovered binding wins;
 * otherwise we passively show the LMB skill (if any).
 */
interface ActiveDescriptor {
  range: number;
  kind: SkillTargetingKind;
  passive: boolean;
}

function useActiveDescriptor(): ActiveDescriptor | null {
  const hovered = useGameStore((s) => s.hoveredSkill);
  const mouseLeft = useGameStore((s) => s.mouseLeft);

  return useMemo(() => {
    if (hovered) return { range: hovered.range, kind: hovered.kind, passive: false };
    if (mouseLeft && mouseLeft.kind === "skill") {
      const skill = SKILL_CATALOG.find((s) => s.id === mouseLeft.skillId);
      if (skill && skill.range > 0) {
        return { range: skill.range, kind: targetingKind(skill), passive: true };
      }
    }
    return null;
  }, [hovered, mouseLeft]);
}

/**
 * League-of-Legends-style ground decal showing a skill's effective range.
 *
 * Branches by targeting kind:
 *   - `self`            → no indicator
 *   - `circle`          → ring around the player at radius=range (melee)
 *   - `aoe_around_self` → filled ring at radius=range, warmer color
 *   - `skillshot`       → directional bar from player toward cursor, capped at range
 */
export default function RangeIndicator() {
  const desc = useActiveDescriptor();
  if (!desc) return null;
  if (desc.kind === "self") return null;
  if (desc.kind === "skillshot") return <SkillshotIndicator desc={desc} />;
  return <CircleIndicator desc={desc} />;
}

function CircleIndicator({ desc }: { desc: ActiveDescriptor }) {
  const meshRef = useRef<THREE.Mesh>(null);

  const geometry = useMemo(() => {
    const inner = Math.max(0.05, desc.range - RING_THICKNESS);
    return new THREE.RingGeometry(inner, desc.range, 64);
  }, [desc.range]);

  useFrame(() => {
    if (!meshRef.current) return;
    const { playerX, playerZ } = useGameStore.getState();
    meshRef.current.position.set(playerX, Y_OFFSET, playerZ);
  });

  const color = desc.kind === "aoe_around_self" ? AOE_COLOR : RING_COLOR;
  const baseOpacity = desc.kind === "aoe_around_self" ? 0.6 : 0.55;

  return (
    <mesh ref={meshRef} rotation={[-Math.PI / 2, 0, 0]} renderOrder={1}>
      <primitive object={geometry} attach="geometry" />
      <meshBasicMaterial
        color={color}
        transparent
        opacity={desc.passive ? 0.18 : baseOpacity}
        depthWrite={false}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

function SkillshotIndicator({ desc }: { desc: ActiveDescriptor }) {
  const groupRef = useRef<THREE.Group>(null);
  const barRef = useRef<THREE.Mesh>(null);
  const tipRef = useRef<THREE.Mesh>(null);
  const { camera, pointer } = useThree();

  // Reused scratch objects — avoid GC churn in the frame loop.
  const raycaster = useMemo(() => new THREE.Raycaster(), []);
  const groundPlane = useMemo(() => new THREE.Plane(new THREE.Vector3(0, 1, 0), 0), []);
  const cursor = useMemo(() => new THREE.Vector3(), []);

  useFrame(() => {
    const { playerX, playerZ } = useGameStore.getState();
    if (!groupRef.current || !barRef.current) return;

    raycaster.setFromCamera(pointer, camera);
    const hit = raycaster.ray.intersectPlane(groundPlane, cursor);
    if (!hit) return;

    const dx = cursor.x - playerX;
    const dz = cursor.z - playerZ;
    const dist = Math.hypot(dx, dz);
    if (dist < 1e-4) return;

    const length = Math.min(dist, desc.range);
    const angle = Math.atan2(dx, dz); // rotate around Y so +Z points toward cursor

    groupRef.current.position.set(playerX, Y_OFFSET, playerZ);
    groupRef.current.rotation.set(0, angle, 0);

    // Bar: lies flat on the ground, length along +Z, anchored at player.
    barRef.current.scale.set(SKILLSHOT_WIDTH, length, 1);
    barRef.current.position.set(0, 0, length / 2);

    if (tipRef.current) {
      tipRef.current.position.set(0, 0, length);
    }
  });

  const opacity = desc.passive ? 0.22 : 0.7;

  return (
    <group ref={groupRef} renderOrder={1}>
      {/* Bar — unit-square plane, scaled per-frame so we never rebuild geometry. */}
      <mesh ref={barRef} rotation={[-Math.PI / 2, 0, 0]}>
        <planeGeometry args={[1, 1]} />
        <meshBasicMaterial
          color={SKILLSHOT_COLOR}
          transparent
          opacity={opacity}
          depthWrite={false}
          side={THREE.DoubleSide}
        />
      </mesh>
      {/* Arrow tip at the end of the bar. */}
      <mesh ref={tipRef} rotation={[-Math.PI / 2, 0, 0]}>
        <circleGeometry args={[SKILLSHOT_WIDTH * 0.9, 24]} />
        <meshBasicMaterial
          color={SKILLSHOT_COLOR}
          transparent
          opacity={Math.min(1, opacity + 0.15)}
          depthWrite={false}
          side={THREE.DoubleSide}
        />
      </mesh>
    </group>
  );
}
