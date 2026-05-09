import { useEffect, useRef, useState } from "react";
import { useGameStore } from "../../store/gameStore";

/** Server combat sim runs at 20 Hz, so each cooldown tick = 50 ms. */
const SECS_PER_TICK = 1 / 20;

interface CooldownEntry {
  /** Total cooldown ticks at the moment the skill was triggered. */
  total: number;
  /** `performance.now()` when the cooldown was first observed. */
  startedAt: number;
  /** Last server-reported value, used to detect refreshes. */
  lastServerValue: number;
}

interface CooldownProgress {
  /** 0..1 — fraction of the cooldown still remaining (1 = just cast, 0 = ready). */
  progress: number;
  /** Whole seconds left, rounded up. */
  secondsLeft: number;
}

const READY: CooldownProgress = { progress: 0, secondsLeft: 0 };

/**
 * Smoothly animated cooldown progress for a single skill id.
 *
 * The server ticks cooldowns at 20 Hz, so the displayed sweep is already smooth
 * at the network rate, but we still interpolate locally with `requestAnimationFrame`
 * for a buttery 60 fps render-side animation. When the server-reported cooldown
 * jumps up (a fresh cast), we capture the timestamp and ramp down from there.
 */
export function useCooldownProgress(skillId: string | null): CooldownProgress {
  const entriesRef = useRef<Map<string, CooldownEntry>>(new Map());
  const [tick, setTick] = useState(0);
  const rafRef = useRef<number | null>(null);

  useEffect(() => {
    const loop = () => {
      setTick((t) => (t + 1) % 1_000_000);
      rafRef.current = requestAnimationFrame(loop);
    };
    rafRef.current = requestAnimationFrame(loop);
    return () => {
      if (rafRef.current !== null) cancelAnimationFrame(rafRef.current);
    };
  }, []);

  if (!skillId) return READY;

  const serverValue = useGameStore.getState().skillCooldowns[skillId] ?? 0;
  const entries = entriesRef.current;
  const existing = entries.get(skillId);

  if (serverValue <= 0) {
    if (existing) entries.delete(skillId);
    void tick; // keep React linter happy: depend on tick
    return READY;
  }

  // First time we see this cooldown, or server bumped it (rebuilt from a new cast).
  if (!existing || serverValue > existing.lastServerValue) {
    entries.set(skillId, {
      total: serverValue,
      startedAt: performance.now(),
      lastServerValue: serverValue,
    });
  } else {
    existing.lastServerValue = serverValue;
  }

  const entry = entries.get(skillId)!;
  const elapsedSec = (performance.now() - entry.startedAt) / 1000;
  const totalSec = entry.total * SECS_PER_TICK;
  const remainingSec = Math.max(0, totalSec - elapsedSec);
  const progress = totalSec > 0 ? remainingSec / totalSec : 0;
  void tick;

  return {
    progress,
    secondsLeft: Math.ceil(remainingSec),
  };
}
