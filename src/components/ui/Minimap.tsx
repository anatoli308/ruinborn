import { useCallback, useEffect, useRef } from "react";
import { useGameStore } from "../../store/gameStore";

const MINIMAP_SIZE = 140;
const WORLD_HALF = 100; // world coordinates run -100..100

/** Mini-map showing player, markets, enemies and loot. Waypoint stones are clickable in the 3D world, not here. */
export default function Minimap() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const playerX = useGameStore((s) => s.playerX);
  const playerZ = useGameStore((s) => s.playerZ);
  const playerMarkets = useGameStore((s) => s.playerMarkets);
  const enemies = useGameStore((s) => s.enemies);
  const lootDrops = useGameStore((s) => s.lootDrops);
  const zoneId = useGameStore((s) => s.zone);
  const zones = useGameStore((s) => s.zones);

  const currentZone = zones.find((z) => z.id === zoneId);
  const zoneLabel = currentZone?.name ?? zoneId;

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = canvas.width;
    const h = canvas.height;
    const scale = w / (WORLD_HALF * 2);

    ctx.fillStyle = "#0a0e17";
    ctx.fillRect(0, 0, w, h);

    // Zone overlays — D2-style colored areas so the player can read the world.
    const overlays: Array<{ minX: number; maxX: number; minZ: number; maxZ: number; fill: string }> = [
      { minX: -30, maxX: 30, minZ: -30, maxZ: 30, fill: "rgba(180, 120, 60, 0.30)" },
      { minX: -60, maxX: 60, minZ: 30, maxZ: 90, fill: "rgba(80, 140, 70, 0.18)" },
      { minX: 60, maxX: 120, minZ: 30, maxZ: 120, fill: "rgba(120, 70, 120, 0.20)" },
    ];
    for (const z of overlays) {
      const x = (z.minX + WORLD_HALF) * scale;
      const y = (z.minZ + WORLD_HALF) * scale;
      const zw = (z.maxX - z.minX) * scale;
      const zh = (z.maxZ - z.minZ) * scale;
      ctx.fillStyle = z.fill;
      ctx.fillRect(x, y, zw, zh);
    }

    // Town wall outline.
    ctx.strokeStyle = "#b08840";
    ctx.lineWidth = 1;
    ctx.strokeRect((-30 + WORLD_HALF) * scale, (-30 + WORLD_HALF) * scale, 60 * scale, 60 * scale);

    // Enemies
    for (const e of enemies) {
      if (e.state === "dead") continue;
      const ex = (e.x + WORLD_HALF) * scale;
      const ez = (e.z + WORLD_HALF) * scale;
      ctx.fillStyle = "#e74c3c";
      ctx.fillRect(ex - 1, ez - 1, 3, 3);
    }

    // Loot drops
    for (const l of lootDrops) {
      const lx = (l.x + WORLD_HALF) * scale;
      const lz = (l.z + WORLD_HALF) * scale;
      ctx.fillStyle = "#ffaa00";
      ctx.fillRect(lx, lz, 2, 2);
    }

    // Player markets
    for (const m of playerMarkets) {
      const mx = (m.x + WORLD_HALF) * scale;
      const mz = (m.z + WORLD_HALF) * scale;
      ctx.fillStyle = "#ffd700";
      ctx.fillRect(mx - 2, mz - 2, 5, 5);
    }

    // Player
    const px = (playerX + WORLD_HALF) * scale;
    const pz = (playerZ + WORLD_HALF) * scale;
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
      <div className="hud-minimap__zone">{zoneLabel}</div>
      <canvas
        ref={canvasRef}
        width={MINIMAP_SIZE}
        height={MINIMAP_SIZE}
        className="rounded border border-gray-700 bg-black/80"
      />
    </div>
  );
}
