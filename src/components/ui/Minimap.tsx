import { useCallback, useEffect, useRef } from "react";
import { useGameStore } from "../../store/gameStore";

/** Mini-map showing player and trading post positions */
export default function Minimap() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const playerX = useGameStore((s) => s.playerX);
  const playerZ = useGameStore((s) => s.playerZ);
  const playerMarkets = useGameStore((s) => s.playerMarkets);
  const enemies = useGameStore((s) => s.enemies);
  const lootDrops = useGameStore((s) => s.lootDrops);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = canvas.width;
    const h = canvas.height;
    const scale = w / 200; // world = -100..100

    ctx.fillStyle = "#0a0e17";
    ctx.fillRect(0, 0, w, h);

    // Zone overlays — D2-style colored areas so the player can read the world.
    const zones: Array<{ minX: number; maxX: number; minZ: number; maxZ: number; fill: string; label: string }> = [
      { minX: -30, maxX: 30, minZ: -30, maxZ: 30, fill: "rgba(180, 120, 60, 0.30)", label: "Stadt" },
      { minX: -60, maxX: 60, minZ: 30,  maxZ: 90, fill: "rgba(80, 140, 70, 0.18)", label: "Wildnis" },
      { minX: 60,  maxX: 120, minZ: 30, maxZ: 120, fill: "rgba(120, 70, 120, 0.20)", label: "Gräberfeld" },
    ];
    for (const z of zones) {
      const x = (z.minX + 100) * scale;
      const y = (z.minZ + 100) * scale;
      const zw = (z.maxX - z.minX) * scale;
      const zh = (z.maxZ - z.minZ) * scale;
      ctx.fillStyle = z.fill;
      ctx.fillRect(x, y, zw, zh);
    }

    // Town wall outline.
    ctx.strokeStyle = "#b08840";
    ctx.lineWidth = 1;
    ctx.strokeRect((-30 + 100) * scale, (-30 + 100) * scale, 60 * scale, 60 * scale);

    // Enemies
    for (const e of enemies) {
      if (e.state === "dead") continue;
      const ex = (e.x + 100) * scale;
      const ez = (e.z + 100) * scale;
      ctx.fillStyle = "#e74c3c";
      ctx.fillRect(ex - 1, ez - 1, 3, 3);
    }

    // Loot drops
    for (const l of lootDrops) {
      const lx = (l.x + 100) * scale;
      const lz = (l.z + 100) * scale;
      ctx.fillStyle = "#ffaa00";
      ctx.fillRect(lx, lz, 2, 2);
    }

    // Player markets
    for (const m of playerMarkets) {
      const mx = (m.x + 100) * scale;
      const mz = (m.z + 100) * scale;
      ctx.fillStyle = "#ffd700";
      ctx.fillRect(mx - 2, mz - 2, 5, 5);
    }

    // Waypoint stones (one per zone — must match world.rs).
    const waypoints = [
      { x: 0,  z: 0,  color: "#3b82f6" },
      { x: 0,  z: 60, color: "#3b82f6" },
      { x: 90, z: 75, color: "#3b82f6" },
    ];
    for (const wp of waypoints) {
      const wx = (wp.x + 100) * scale;
      const wz = (wp.z + 100) * scale;
      ctx.fillStyle = wp.color;
      ctx.beginPath();
      ctx.arc(wx, wz, 2.5, 0, Math.PI * 2);
      ctx.fill();
    }

    // Player
    const px = (playerX + 100) * scale;
    const pz = (playerZ + 100) * scale;
    ctx.fillStyle = "#00ff00";
    ctx.beginPath();
    ctx.arc(px, pz, 3, 0, Math.PI * 2);
    ctx.fill();

    // Border
    ctx.strokeStyle = "#333";
    ctx.strokeRect(0, 0, w, h);
  }, [playerX, playerZ, playerMarkets, enemies, lootDrops]);

  useEffect(() => {
    draw();
  }, [draw]);

  return (
    <div className="hud-minimap">
      <canvas
        ref={canvasRef}
        width={140}
        height={140}
        className="rounded border border-gray-700 bg-black/80"
      />
    </div>
  );
}
