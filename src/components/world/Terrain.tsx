import { useMemo } from "react";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";

const GRID_SPACING = 10;
const WORLD_BOUND = 90;
const GRID_COUNT = (WORLD_BOUND * 2) / GRID_SPACING + 1; // 19

// ── Town zone bounds (must match crates/tradewars-game/src/world.rs) ─────
const TOWN_MIN_X = -30;
const TOWN_MAX_X = 30;
const TOWN_MIN_Z = -30;
const TOWN_MAX_Z = 30;
const TOWN_W = TOWN_MAX_X - TOWN_MIN_X; // 60
const TOWN_D = TOWN_MAX_Z - TOWN_MIN_Z; // 60
const TOWN_CX = (TOWN_MIN_X + TOWN_MAX_X) / 2; // 0
const TOWN_CZ = (TOWN_MIN_Z + TOWN_MAX_Z) / 2; // 0
/** Width of the gate opening on the south wall (toward Wilderness). */
const GATE_WIDTH = 6;
const WALL_HEIGHT = 1.6;
const WALL_THICKNESS = 0.6;

/** Clean flat grid world — each intersection can hold a player market. */
export default function Terrain() {
  const playerMarkets = useGameStore((s) => s.playerMarkets);

  const occupiedSet = useMemo(() => {
    const s = new Set<string>();
    for (const m of playerMarkets) {
      s.add(`${m.x},${m.z}`);
    }
    return s;
  }, [playerMarkets]);

  // Grid point positions (only the dots, not occupied slots — markets render separately)
  const dots = useMemo(() => {
    const pts: { x: number; z: number }[] = [];
    for (let i = 0; i < GRID_COUNT; i++) {
      for (let j = 0; j < GRID_COUNT; j++) {
        const x = -WORLD_BOUND + i * GRID_SPACING;
        const z = -WORLD_BOUND + j * GRID_SPACING;
        pts.push({ x, z });
      }
    }
    return pts;
  }, []);

  const dotGeo = useMemo(() => new THREE.CircleGeometry(0.3, 6), []);

  // South-wall gate: split into two segments on either side of the opening.
  const southSegLen = (TOWN_W - GATE_WIDTH) / 2;
  const southSegOffset = GATE_WIDTH / 2 + southSegLen / 2;

  return (
    <>
      {/* Flat ground (full world) */}
      <mesh rotation={[-Math.PI / 2, 0, 0]} receiveShadow>
        <planeGeometry args={[200, 200]} />
        <meshStandardMaterial color="#1a1e2a" roughness={1} />
      </mesh>

      {/* Town floor — warm tone so the safe zone is obvious. */}
      <mesh
        rotation={[-Math.PI / 2, 0, 0]}
        position={[TOWN_CX, 0.015, TOWN_CZ]}
        receiveShadow
      >
        <planeGeometry args={[TOWN_W, TOWN_D]} />
        <meshStandardMaterial color="#3a2e1f" roughness={0.95} />
      </mesh>

      {/* Town walls — Rogue-Encampment style stone perimeter. */}
      {/* North wall (top, full length) */}
      <mesh
        position={[TOWN_CX, WALL_HEIGHT / 2, TOWN_MIN_Z]}
        castShadow
        receiveShadow
      >
        <boxGeometry args={[TOWN_W, WALL_HEIGHT, WALL_THICKNESS]} />
        <meshStandardMaterial color="#5b4a3a" roughness={0.9} />
      </mesh>
      {/* West wall */}
      <mesh
        position={[TOWN_MIN_X, WALL_HEIGHT / 2, TOWN_CZ]}
        castShadow
        receiveShadow
      >
        <boxGeometry args={[WALL_THICKNESS, WALL_HEIGHT, TOWN_D]} />
        <meshStandardMaterial color="#5b4a3a" roughness={0.9} />
      </mesh>
      {/* East wall */}
      <mesh
        position={[TOWN_MAX_X, WALL_HEIGHT / 2, TOWN_CZ]}
        castShadow
        receiveShadow
      >
        <boxGeometry args={[WALL_THICKNESS, WALL_HEIGHT, TOWN_D]} />
        <meshStandardMaterial color="#5b4a3a" roughness={0.9} />
      </mesh>
      {/* South wall — split around the gate opening (toward Wilderness, +Z). */}
      <mesh
        position={[TOWN_CX - southSegOffset, WALL_HEIGHT / 2, TOWN_MAX_Z]}
        castShadow
        receiveShadow
      >
        <boxGeometry args={[southSegLen, WALL_HEIGHT, WALL_THICKNESS]} />
        <meshStandardMaterial color="#5b4a3a" roughness={0.9} />
      </mesh>
      <mesh
        position={[TOWN_CX + southSegOffset, WALL_HEIGHT / 2, TOWN_MAX_Z]}
        castShadow
        receiveShadow
      >
        <boxGeometry args={[southSegLen, WALL_HEIGHT, WALL_THICKNESS]} />
        <meshStandardMaterial color="#5b4a3a" roughness={0.9} />
      </mesh>
      {/* Gate posts — golden pillars marking the exit. */}
      <mesh position={[TOWN_CX - GATE_WIDTH / 2, WALL_HEIGHT / 2 + 0.4, TOWN_MAX_Z]} castShadow>
        <boxGeometry args={[0.5, WALL_HEIGHT + 0.8, 0.5]} />
        <meshStandardMaterial color="#b08840" emissive="#553311" emissiveIntensity={0.4} />
      </mesh>
      <mesh position={[TOWN_CX + GATE_WIDTH / 2, WALL_HEIGHT / 2 + 0.4, TOWN_MAX_Z]} castShadow>
        <boxGeometry args={[0.5, WALL_HEIGHT + 0.8, 0.5]} />
        <meshStandardMaterial color="#b08840" emissive="#553311" emissiveIntensity={0.4} />
      </mesh>

      {/* Town waypoint stone (center of the city). */}
      <mesh position={[0, 0.6, 0]} castShadow>
        <cylinderGeometry args={[0.7, 0.9, 1.2, 6]} />
        <meshStandardMaterial color="#3b82f6" emissive="#1d4ed8" emissiveIntensity={0.6} />
      </mesh>

      {/* Grid lines (covers the wilderness area). */}
      <gridHelper
        args={[WORLD_BOUND * 2, (WORLD_BOUND * 2) / GRID_SPACING, "#2a2e3a", "#222638"]}
        position={[0, 0.01, 0]}
      />

      {/* Grid dots (skip dots inside the town footprint to keep the city clean). */}
      {dots.map((d) => {
        const inTown =
          d.x >= TOWN_MIN_X && d.x <= TOWN_MAX_X && d.z >= TOWN_MIN_Z && d.z <= TOWN_MAX_Z;
        if (inTown) return null;
        const key = `${d.x},${d.z}`;
        const occupied = occupiedSet.has(key);
        return (
          <mesh
            key={key}
            rotation={[-Math.PI / 2, 0, 0]}
            position={[d.x, 0.02, d.z]}
            geometry={dotGeo}
          >
            <meshBasicMaterial color={occupied ? "#ffd700" : "#3a3e4a"} />
          </mesh>
        );
      })}
    </>
  );
}
