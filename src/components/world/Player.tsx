import { useEffect, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";
import FlareSprite, { angleToFlareDirection } from "./FlareSprite";

const SPEED = 7;
const WORLD_BOUND = 90;
const MAX_FRAME_DELTA = 0.1;
/** How fast the predicted position converges to the server position */
const RECONCILE_LERP = 0.15;
const RECONCILE_SNAP_DISTANCE = 8;

/** Render order: back layers first so depth-sort + alphaTest stack cleanly. */
const PLAYER_LAYERS = [
  "default_legs",
  "default_feet",
  "default_chest",
  "default_hands",
  "head_short",
];

/** Player character with WASD movement — client-side predicted, server-reconciled */
export default function Player() {
  const group = useRef<THREE.Group>(null!);
  const keys = useRef<Record<string, boolean>>({});

  // Sprite animation + facing — refs so FlareSprite reads them without re-renders.
  const animationRef = useRef("stance");
  const directionRef = useRef(0);
  const facingRef = useRef(0);

  const sendMove = useGameStore((s) => s.sendMove);
  const sendToggleTradePanel = useGameStore((s) => s.sendToggleTradePanel);
  const sendCreateMarket = useGameStore((s) => s.sendCreateMarket);
  const sendUseActionSlot = useGameStore((s) => s.sendUseActionSlot);
  const toggleInventory = useGameStore((s) => s.toggleInventory);
  const toggleCharacter = useGameStore((s) => s.toggleCharacter);
  const toggleSkillTree = useGameStore((s) => s.toggleSkillTree);
  const cycleTarget = useGameStore((s) => s.cycleTarget);
  const setTargetEnemy = useGameStore((s) => s.setTargetEnemy);
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
      if (e.code === "Tab") {
        // WoW-style: cycle the closest alive enemy in the current zone.
        e.preventDefault();
        cycleTarget();
      }
      if (e.code === "Escape") {
        setTargetEnemy(null);
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
  }, [sendCreateMarket, sendToggleTradePanel, sendUseActionSlot, toggleInventory, toggleCharacter, toggleSkillTree, cycleTarget, setTargetEnemy]);

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

      // Track facing for sprite direction (group itself is NOT rotated — the
      // FlareSprite billboards toward the camera independently).
      facingRef.current = Math.atan2(dx, -dz);
      animationRef.current = "run";
    } else {
      animationRef.current = "stance";
    }

    directionRef.current = angleToFlareDirection(facingRef.current);

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

    // Apply predicted position to mesh — small Y lift keeps the sprite's
    // bottom rows above the ground plane (FLARE pivot sits a few px above
    // the frame's bottom edge, so the mesh extends slightly below origin).
    group.current.position.set(predictedPos.current.x, 0.2, predictedPos.current.z);
  });

  return (
    <group ref={group} name="local-player">
      <FlareSprite
        layers={PLAYER_LAYERS}
        animationRef={animationRef}
        directionRef={directionRef}
      />
    </group>
  );
}
