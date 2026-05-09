import { useEffect, useState } from "react";
import { useGameStore } from "../../store/gameStore";

const TOAST_DURATION_MS = 2200;
const TOAST_FADE_MS = 350;

/**
 * Bottom-center transient banner for failed actions ("Target out of range", "Not enough mana", …).
 *
 * Driven entirely by `lastError` in the store, which is set whenever the server
 * answers an action with `success: false`. The component schedules its own
 * cleanup so failed actions do not leak into the next session.
 */
export default function Toast() {
  const lastError = useGameStore((s) => s.lastError);
  const clearLastError = useGameStore((s) => s.clearLastError);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (!lastError) {
      setVisible(false);
      return;
    }
    setVisible(true);
    const fadeTimer = window.setTimeout(() => setVisible(false), TOAST_DURATION_MS);
    const clearTimer = window.setTimeout(() => clearLastError(), TOAST_DURATION_MS + TOAST_FADE_MS);
    return () => {
      window.clearTimeout(fadeTimer);
      window.clearTimeout(clearTimer);
    };
  }, [lastError, clearLastError]);

  if (!lastError) return null;

  return (
    <div
      className="toast"
      style={{
        opacity: visible ? 1 : 0,
        transform: visible ? "translate(-50%, 0)" : "translate(-50%, 8px)",
        transition: `opacity ${TOAST_FADE_MS}ms ease, transform ${TOAST_FADE_MS}ms ease`,
      }}
    >
      {lastError.message}
    </div>
  );
}
