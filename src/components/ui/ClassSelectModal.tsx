import { useState } from "react";
import { useGameStore } from "../../store/gameStore";
import { CLASS_CATALOG } from "../../data/classes";
import type { ClassId } from "../../types";

/**
 * Class selection modal — shown automatically once on first connect when
 * the server reports `class_id === null`. Cannot be dismissed.
 */
export default function ClassSelectModal() {
  const classId = useGameStore((s) => s.classId);
  const connected = useGameStore((s) => s.connected);
  const sendChooseClass = useGameStore((s) => s.sendChooseClass);
  const [picked, setPicked] = useState<ClassId | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");

  if (!connected || classId !== null) return null;

  const handleConfirm = async () => {
    if (!picked || submitting) return;
    setSubmitting(true);
    setError("");
    const result = await sendChooseClass(picked);
    if (!result.success) {
      setError(result.message);
      setSubmitting(false);
    }
  };

  return (
    <div className="trade-overlay" style={{ zIndex: 100 }}>
      <div
        className="trade-panel"
        style={{ width: 760, maxWidth: "90vw", padding: 24 }}
      >
        <div style={{ textAlign: "center", marginBottom: 20 }}>
          <h2 style={{ margin: 0, color: "#facc15", fontSize: 24 }}>
            ⚔ Wähle deine Klasse
          </h2>
          <p style={{ margin: "6px 0 0", color: "#9ca3af", fontSize: 13 }}>
            Diese Wahl ist permanent und definiert deine Basisstats und
            Starter-Fertigkeiten.
          </p>
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(3, 1fr)",
            gap: 12,
            marginBottom: 20,
          }}
        >
          {CLASS_CATALOG.map((c) => {
            const isPicked = picked === c.id;
            return (
              <button
                key={c.id}
                onClick={() => setPicked(c.id)}
                style={{
                  padding: 16,
                  background: isPicked ? "#1f2937" : "#0b1220",
                  border: `2px solid ${isPicked ? "#facc15" : "#374151"}`,
                  borderRadius: 8,
                  cursor: "pointer",
                  color: "#e5e7eb",
                  textAlign: "left",
                  transition: "border-color 120ms",
                }}
              >
                <div style={{ fontSize: 36, marginBottom: 6 }}>{c.icon}</div>
                <div style={{ fontSize: 16, fontWeight: 700, color: "#facc15" }}>
                  {c.name}
                </div>
                <div style={{ fontSize: 12, color: "#9ca3af", marginBottom: 10 }}>
                  {c.tagline}
                </div>
                <div style={{ fontSize: 11, lineHeight: 1.6 }}>
                  <div>💪 STR {c.baseStats.strength}</div>
                  <div>🏹 DEX {c.baseStats.dexterity}</div>
                  <div>❤ VIT {c.baseStats.vitality}</div>
                  <div>✨ ENE {c.baseStats.energy}</div>
                </div>
                <div style={{ fontSize: 11, marginTop: 8, color: "#a78bfa" }}>
                  Start: {c.starterSkills.join(", ")}
                </div>
              </button>
            );
          })}
        </div>

        {error && (
          <div style={{ color: "#f87171", textAlign: "center", marginBottom: 12 }}>
            {error}
          </div>
        )}

        <div style={{ textAlign: "center" }}>
          <button
            onClick={handleConfirm}
            disabled={!picked || submitting}
            style={{
              padding: "10px 32px",
              background: !picked || submitting ? "#374151" : "#facc15",
              color: !picked || submitting ? "#9ca3af" : "#111827",
              border: "none",
              borderRadius: 6,
              fontSize: 14,
              fontWeight: 700,
              cursor: !picked || submitting ? "not-allowed" : "pointer",
            }}
          >
            {submitting ? "..." : "Klasse bestätigen"}
          </button>
        </div>
      </div>
    </div>
  );
}
