import { useCallback, useEffect, useRef } from "react";
import { useGameStore } from "../../store/gameStore";

/** Mini-map showing player and trading post positions */
export default function Minimap() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const playerX = useGameStore((s) => s.playerX);
  const playerZ = useGameStore((s) => s.playerZ);
  const tradingPosts = useGameStore((s) => s.tradingPosts);

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

    // Trading posts
    for (const p of tradingPosts) {
      const px = (p.x + 100) * scale;
      const pz = (p.z + 100) * scale;
      ctx.fillStyle = p.owned ? "#ffd700" : "#8b6914";
      ctx.fillRect(px - 2, pz - 2, 5, 5);
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
  }, [playerX, playerZ, tradingPosts]);

  useEffect(() => {
    draw();
  }, [draw]);

  return (
    <div className="absolute bottom-3 right-3 z-10 pointer-events-none">
      <canvas
        ref={canvasRef}
        width={140}
        height={140}
        className="rounded border border-gray-700 bg-black/80"
      />
    </div>
  );
}
