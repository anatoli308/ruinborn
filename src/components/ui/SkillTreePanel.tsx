import { useState } from "react";
import { useGameStore } from "../../store/gameStore";
import { SKILL_CATALOG, classInfo } from "../../data/classes";
import SkillIconView from "./SkillIconView";
import type { SkillDef } from "../../types";

const EFFECT_LABEL: Record<SkillDef["effect"], string> = {
  direct_damage: "Direct damage",
  aoe_around: "AoE",
  damage_over_time: "DoT",
  teleport: "Movement",
  self_buff: "Self buff",
  placeholder: "Placeholder",
};

/**
 * Skill tree panel — toggled with [K]. Shows the player's class skills,
 * lets them spend unspent skill points, and casts skills via [B].
 *
 * Casting from the panel uses the nearest enemy as target for direct-damage
 * skills, the player's current position for AoE/buffs, and (placeholder) skips
 * skills that need an explicit ground click.
 */
export default function SkillTreePanel() {
  const open = useGameStore((s) => s.skillTreeOpen);
  const setOpen = useGameStore((s) => s.setSkillTreeOpen);
  const classId = useGameStore((s) => s.classId);
  const level = useGameStore((s) => s.level);
  const allocated = useGameStore((s) => s.allocatedSkills);
  const unspent = useGameStore((s) => s.unspentSkillPoints);
  const cooldowns = useGameStore((s) => s.skillCooldowns);
  const buffs = useGameStore((s) => s.activeBuffs);
  const enemies = useGameStore((s) => s.enemies);
  const playerX = useGameStore((s) => s.playerX);
  const playerZ = useGameStore((s) => s.playerZ);
  const sendAllocateSkill = useGameStore((s) => s.sendAllocateSkill);
  const sendCastSkill = useGameStore((s) => s.sendCastSkill);
  const sendSetActionSlotSkill = useGameStore((s) => s.sendSetActionSlotSkill);
  const sendBindMouseSkill = useGameStore((s) => s.sendBindMouseSkill);
  const [feedback, setFeedback] = useState("");

  if (!open || !classId) return null;

  const info = classInfo(classId);
  const skills = SKILL_CATALOG.filter((s) => s.classId === classId);

  const nearestEnemyId = (): string | null => {
    let bestId: string | null = null;
    let bestD2 = Infinity;
    for (const e of enemies) {
      if (e.state === "dead") continue;
      const dx = e.x - playerX;
      const dz = e.z - playerZ;
      const d2 = dx * dx + dz * dz;
      if (d2 < bestD2) {
        bestD2 = d2;
        bestId = e.id;
      }
    }
    return bestId;
  };

  const allocate = async (id: string) => {
    const r = await sendAllocateSkill(id);
    setFeedback(r.message);
  };

  const cast = async (def: SkillDef) => {
    let targetEnemyId: string | null = null;
    let tx: number | null = null;
    let tz: number | null = null;
    if (def.effect === "direct_damage" || def.effect === "damage_over_time") {
      targetEnemyId = nearestEnemyId();
      if (!targetEnemyId) {
        setFeedback("No target in range.");
        return;
      }
    } else if (def.effect === "teleport") {
      // Without map click, teleport in the player's facing direction (forward Z+).
      tx = playerX;
      tz = playerZ + Math.min(def.range, 6);
    }
    const r = await sendCastSkill(def.id, targetEnemyId, tx, tz);
    setFeedback(r.message);
  };

  return (
    <div className="trade-overlay" style={{ zIndex: 60 }}>
      <div
        className="trade-panel"
        style={{ width: 720, maxWidth: "92vw", padding: 20 }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: 14,
          }}
        >
          <div>
            <h2 style={{ margin: 0, color: "#facc15", fontSize: 20 }}>
              {info.icon} Fertigkeiten — {info.name}
            </h2>
            <div style={{ fontSize: 12, color: "#9ca3af", marginTop: 2 }}>
              Level {level} · Skill points:{" "}
              <span style={{ color: "#facc15", fontWeight: 700 }}>
                {unspent}
              </span>
            </div>
          </div>
          <button
            onClick={() => setOpen(false)}
            className="tp-close"
            aria-label="Schliessen"
          >
            ✕
          </button>
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(3, 1fr)",
            gap: 10,
          }}
        >
          {skills.map((sk) => {
            const lvl = allocated[sk.id] ?? 0;
            const cd = cooldowns[sk.id] ?? 0;
            const buff = buffs[sk.id] ?? 0;
            const locked = level < sk.requiresLevel;
            const known = lvl > 0 || info.starterSkills.includes(sk.id);
            return (
              <div
                key={sk.id}
                draggable={known && !locked}
                onDragStart={(e) => {
                  if (!known || locked) return;
                  e.dataTransfer.setData(
                    "application/x-ruinborn-skill",
                    JSON.stringify({ skillId: sk.id }),
                  );
                  e.dataTransfer.effectAllowed = "copy";
                }}
                style={{
                  padding: 12,
                  background: locked ? "#0b1220" : "#111827",
                  border: `1px solid ${known ? "#a78bfa" : "#374151"}`,
                  borderRadius: 6,
                  opacity: locked ? 0.5 : 1,
                  cursor: known && !locked ? "grab" : "default",
                }}
              >
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "baseline",
                  }}
                >
                  <div style={{ fontWeight: 700, color: "#e5e7eb", display: "flex", alignItems: "center", gap: 6 }}>
                    {sk.icon ? (
                      <SkillIconView icon={sk.icon} className="skill-card__icon" alt={sk.name} />
                    ) : null}
                    <span>{sk.name}</span>
                  </div>
                  <div style={{ fontSize: 11, color: "#9ca3af" }}>
                    Level {lvl}
                  </div>
                </div>
                <div style={{ fontSize: 11, color: "#9ca3af", marginBottom: 4 }}>
                  {EFFECT_LABEL[sk.effect]} · requires Lv {sk.requiresLevel}
                </div>
                <div style={{ fontSize: 12, color: "#cbd5e1", minHeight: 32 }}>
                  {sk.description}
                </div>
                <div style={{ fontSize: 11, color: "#9ca3af", marginTop: 4 }}>
                  💧 {sk.manaCost} · CD {sk.cooldownTicks}t · Range {sk.range}
                  {cd > 0 && (
                    <span style={{ color: "#f87171" }}> · CD {cd}t</span>
                  )}
                  {buff > 0 && (
                    <span style={{ color: "#34d399" }}> · Buff {buff}t</span>
                  )}
                </div>
                <div style={{ display: "flex", gap: 6, marginTop: 8 }}>
                  <button
                    onClick={() => allocate(sk.id)}
                    disabled={locked || unspent <= 0}
                    style={{
                      flex: 1,
                      padding: "4px 8px",
                      fontSize: 11,
                      background:
                        locked || unspent <= 0 ? "#374151" : "#a78bfa",
                      color:
                        locked || unspent <= 0 ? "#9ca3af" : "#0b1220",
                      border: "none",
                      borderRadius: 4,
                      cursor:
                        locked || unspent <= 0 ? "not-allowed" : "pointer",
                      fontWeight: 700,
                    }}
                  >
                    + Punkt
                  </button>
                  <button
                    onClick={() => cast(sk)}
                    disabled={locked || (!known) || cd > 0}
                    style={{
                      flex: 1,
                      padding: "4px 8px",
                      fontSize: 11,
                      background:
                        locked || !known || cd > 0 ? "#374151" : "#facc15",
                      color:
                        locked || !known || cd > 0 ? "#9ca3af" : "#111827",
                      border: "none",
                      borderRadius: 4,
                      cursor:
                        locked || !known || cd > 0 ? "not-allowed" : "pointer",
                      fontWeight: 700,
                    }}
                  >
                    Wirken
                  </button>
                </div>
                {known && !locked && (
                  <div
                    style={{
                      display: "flex",
                      gap: 4,
                      marginTop: 6,
                      flexWrap: "wrap",
                      alignItems: "center",
                    }}
                  >
                    <span style={{ fontSize: 10, color: "#9ca3af" }}>
                      Auf Hotbar:
                    </span>
                    {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((slot) => (
                      <button
                        key={slot}
                        onClick={async () => {
                          const r = await sendSetActionSlotSkill(slot - 1, sk.id);
                          setFeedback(r.message || `Auf Slot ${slot} gelegt.`);
                        }}
                        title={`${sk.name} → Hotkey ${slot}`}
                        style={{
                          width: 22,
                          height: 22,
                          padding: 0,
                          fontSize: 10,
                          background: "#1f2937",
                          color: "#e5e7eb",
                          border: "1px solid #374151",
                          borderRadius: 3,
                          cursor: "pointer",
                          fontWeight: 700,
                        }}
                      >
                        {slot}
                      </button>
                    ))}
                    <span style={{ fontSize: 10, color: "#9ca3af", marginLeft: 6 }}>
                      Mouse:
                    </span>
                    {([
                      { label: "L", btn: 0 as const, title: "Left click" },
                      { label: "R", btn: 1 as const, title: "Right click" },
                    ]).map((m) => (
                      <button
                        key={m.label}
                        onClick={async () => {
                          const r = await sendBindMouseSkill(m.btn, sk.id);
                          setFeedback(r.message || `${m.title}: ${sk.name}`);
                        }}
                        title={`${sk.name} → ${m.title}`}
                        style={{
                          width: 26,
                          height: 22,
                          padding: 0,
                          fontSize: 10,
                          background: "#3a2b14",
                          color: "#facc15",
                          border: "1px solid #b08840",
                          borderRadius: 3,
                          cursor: "pointer",
                          fontWeight: 700,
                        }}
                      >
                        {m.label}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {feedback && (
          <div
            style={{
              marginTop: 12,
              fontSize: 12,
              color: "#cbd5e1",
              textAlign: "center",
            }}
          >
            {feedback}
          </div>
        )}
      </div>
    </div>
  );
}
