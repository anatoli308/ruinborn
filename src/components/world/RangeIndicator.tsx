import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";
import { SKILL_CATALOG } from "../../data/classes";

const Y_OFFSET = 0.05;
const RING_THICKNESS = 0.18;
const RING_COLOR = "#7ec8ff";

/**
 * League-of-Legends-style ground decal showing a skill's effective range.
 *
 * Strategy:
 * - When the player hovers an action-bar slot, `hoveredSkillRange` is set in the store
 *   and the ring renders at that radius.
 * - Otherwise, if a skill is bound to LMB, its range is shown faintly so the player
 *   always knows their primary attack reach.
 *
 * The mesh uses a thin ring geometry on the XZ plane and follows the player every frame
 * to stay visually attached without re-renders.
 */
export default function RangeIndicator() {
  const meshRef = useRef<THREE.Mesh>(null);
  const hoveredSkillRange = useGameStore((s) => s.hoveredSkillRange);
  const mouseLeft = useGameStore((s) => s.mouseLeft);

  const lmbRange = useMemo(() => {
    if (!mouseLeft || mouseLeft.kind !== "skill") return null;
    const skill = SKILL_CATALOG.find((s) => s.id === mouseLeft.skillId);
    return skill && skill.range > 0 ? skill.range : null;
  }, [mouseLeft]);

  // Pick which range to render (hovered wins, then passive LMB).
  const activeRange = hoveredSkillRange ?? lmbRange;
  const isPassive = hoveredSkillRange === null && lmbRange !== null;

  // Rebuild ring geometry only when the radius actually changes.
  const geometry = useMemo(() => {
    if (activeRange === null) return null;
    const inner = Math.max(0.05, activeRange - RING_THICKNESS);
    const outer = activeRange;
    return new THREE.RingGeometry(inner, outer, 64);
  }, [activeRange]);

  useFrame(() => {
    if (!meshRef.current) return;
    const { playerX, playerZ } = useGameStore.getState();
    meshRef.current.position.set(playerX, Y_OFFSET, playerZ);
  });

  if (geometry === null) return null;

  return (
    <mesh ref={meshRef} rotation={[-Math.PI / 2, 0, 0]} renderOrder={1}>
      <primitive object={geometry} attach="geometry" />
      <meshBasicMaterial
        color={RING_COLOR}
        transparent
        opacity={isPassive ? 0.18 : 0.55}
        depthWrite={false}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}
