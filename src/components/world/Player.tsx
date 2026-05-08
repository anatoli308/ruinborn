import { useEffect, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";

const SPEED = 12;
const WORLD_BOUND = 90;
const MAX_FRAME_DELTA = 0.1;
/** How fast the predicted position converges to the server position */
const RECONCILE_LERP = 0.15;
const RECONCILE_SNAP_DISTANCE = 8;

/** Player character with WASD movement — client-side predicted, server-reconciled */
export default function Player() {
  const group = useRef<THREE.Group>(null!);
  const indicator = useRef<THREE.Mesh>(null!);
  const keys = useRef<Record<string, boolean>>({});

  const sendMove = useGameStore((s) => s.sendMove);
  const sendToggleTradePanel = useGameStore((s) => s.sendToggleTradePanel);
  const sendCreateMarket = useGameStore((s) => s.sendCreateMarket);
  const sendUseActionSlot = useGameStore((s) => s.sendUseActionSlot);
  const toggleInventory = useGameStore((s) => s.toggleInventory);
  const toggleCharacter = useGameStore((s) => s.toggleCharacter);
  const toggleSkillTree = useGameStore((s) => s.toggleSkillTree);
  const showTradePanel = useGameStore((s) => s.showTradePanel);

  // Predicted local position (renders immediately, reconciles with server)
  const predictedPos = useRef({
    x: useGameStore.getState().playerX,
    z: useGameStore.getState().playerZ,
  });

  // Register keyboard listeners once and clean up on unmount (StrictMode-safe)
  useEffect(() => {
    const onDown = (e: KeyboardEvent) => {
      // Block hotkeys while typing in form fields.
      const target = e.target as HTMLElement | null;
      if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) {
        return;
      }
      keys.current[e.code] = true;
      // [T] = toggle the trading-post panel (must stand near a market).
      if (e.code === "KeyT") {
        e.preventDefault();
        sendToggleTradePanel();
      }
      if (e.code === "KeyI" || e.code === "KeyB") {
        e.preventDefault();
        toggleInventory();
      }
      if (e.code === "KeyC") {
        e.preventDefault();
        toggleCharacter();
      }
      if (e.code === "KeyK") {
        e.preventDefault();
        toggleSkillTree();
      }
      if (e.code.startsWith("Digit")) {
        const n = Number(e.code.slice(5));
        if (n >= 1 && n <= 9) {
          void sendUseActionSlot(n - 1);
        }
      }
      if (e.code === "KeyM") {
        const name = window.prompt("Name for your market:");
        if (name && name.trim()) sendCreateMarket(name.trim());
      }
    };
    const onUp = (e: KeyboardEvent) => { keys.current[e.code] = false; };
    window.addEventListener("keydown", onDown);
    window.addEventListener("keyup", onUp);
    return () => {
      window.removeEventListener("keydown", onDown);
      window.removeEventListener("keyup", onUp);
    };
  }, [sendCreateMarket, sendToggleTradePanel, sendUseActionSlot, toggleInventory, toggleCharacter, toggleSkillTree]);

  useFrame((_, delta) => {
    if (showTradePanel) return;

    const { playerX, playerZ } = useGameStore.getState();

    let dx = 0;
    let dz = 0;
    const k = keys.current;
    if (k["KeyW"] || k["ArrowUp"]) dz -= 1;
    if (k["KeyS"] || k["ArrowDown"]) dz += 1;
    if (k["KeyA"] || k["ArrowLeft"]) dx -= 1;
    if (k["KeyD"] || k["ArrowRight"]) dx += 1;

    if (dx !== 0 || dz !== 0) {
      const frameDelta = Math.min(delta, MAX_FRAME_DELTA);
      const len = Math.sqrt(dx * dx + dz * dz);
      dx = (dx / len) * SPEED * frameDelta;
      dz = (dz / len) * SPEED * frameDelta;

      // Client-side prediction: apply movement locally immediately
      predictedPos.current.x += dx;
      predictedPos.current.z += dz;

      // Clamp to world bounds (mirror server logic)
      predictedPos.current.x = Math.max(-WORLD_BOUND, Math.min(WORLD_BOUND, predictedPos.current.x));
      predictedPos.current.z = Math.max(-WORLD_BOUND, Math.min(WORLD_BOUND, predictedPos.current.z));

      // Send movement to server
      sendMove(dx, dz);

      // Face direction
      group.current.rotation.y = Math.atan2(dx, -dz);
    }

    // Reconcile: smoothly lerp predicted position toward server-authoritative position
    const errX = playerX - predictedPos.current.x;
    const errZ = playerZ - predictedPos.current.z;
    const distanceToAuthority = Math.hypot(errX, errZ);

    if (distanceToAuthority > RECONCILE_SNAP_DISTANCE) {
      // Large drift indicates teleport/rejoin/out-of-date prediction; snap immediately.
      predictedPos.current.x = playerX;
      predictedPos.current.z = playerZ;
    } else {
      predictedPos.current.x += errX * RECONCILE_LERP;
      predictedPos.current.z += errZ * RECONCILE_LERP;
    }

    // Apply predicted position to mesh
    group.current.position.set(predictedPos.current.x, 0, predictedPos.current.z);

    // Floating indicator animation
    if (indicator.current) {
      indicator.current.rotation.y += delta * 2;
      indicator.current.position.y = 1.6 + Math.sin(Date.now() * 0.003) * 0.1;
    }
  });

  return (
    <group ref={group} name="local-player">
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
