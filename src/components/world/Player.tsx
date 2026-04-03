import { useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";

const SPEED = 12;

/** Player character with WASD movement — sends inputs to dedicated server */
export default function Player() {
  const group = useRef<THREE.Group>(null!);
  const indicator = useRef<THREE.Mesh>(null!);
  const keys = useRef<Record<string, boolean>>({});

  const sendMove = useGameStore((s) => s.sendMove);
  const sendToggleTradePanel = useGameStore((s) => s.sendToggleTradePanel);
  const showTradePanel = useGameStore((s) => s.showTradePanel);

  // Key listeners (registered once via ref check)
  const listenersAttached = useRef(false);
  if (!listenersAttached.current) {
    listenersAttached.current = true;
    const onDown = (e: KeyboardEvent) => {
      keys.current[e.code] = true;
      if (e.code === "KeyE") sendToggleTradePanel();
    };
    const onUp = (e: KeyboardEvent) => { keys.current[e.code] = false; };
    window.addEventListener("keydown", onDown);
    window.addEventListener("keyup", onUp);
  }

  useFrame((_, delta) => {
    if (showTradePanel) return;

    let dx = 0;
    let dz = 0;
    const k = keys.current;
    if (k["KeyW"] || k["ArrowUp"]) dz -= 1;
    if (k["KeyS"] || k["ArrowDown"]) dz += 1;
    if (k["KeyA"] || k["ArrowLeft"]) dx -= 1;
    if (k["KeyD"] || k["ArrowRight"]) dx += 1;

    if (dx !== 0 || dz !== 0) {
      const len = Math.sqrt(dx * dx + dz * dz);
      dx = (dx / len) * SPEED * delta;
      dz = (dz / len) * SPEED * delta;
      // Send movement to Rust server
      sendMove(dx, dz);

      // Face direction
      group.current.rotation.y = Math.atan2(dx, -dz);
    }

    // Sync mesh to store (server-authoritative position)
    const { playerX, playerZ } = useGameStore.getState();
    group.current.position.set(playerX, 0, playerZ);

    // Floating indicator animation
    if (indicator.current) {
      indicator.current.rotation.y += delta * 2;
      indicator.current.position.y = 1.6 + Math.sin(Date.now() * 0.003) * 0.1;
    }
  });

  return (
    <group ref={group}>
      {/* Body */}
      <mesh position={[0, 0.8, 0]} castShadow>
        <capsuleGeometry args={[0.3, 0.6, 4, 8]} />
        <meshStandardMaterial color="#ffd700" roughness={0.4} metalness={0.3} />
      </mesh>
      {/* Head */}
      <mesh position={[0, 1.4, 0]} castShadow>
        <sphereGeometry args={[0.22, 8, 8]} />
        <meshStandardMaterial color="#ffcc88" />
      </mesh>
      {/* Floating indicator */}
      <mesh ref={indicator} position={[0, 1.6, 0]}>
        <octahedronGeometry args={[0.12, 0]} />
        <meshStandardMaterial color="#ffd700" emissive="#ffd700" emissiveIntensity={0.5} />
      </mesh>
    </group>
  );
}
