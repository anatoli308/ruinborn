import { useGameStore } from "../../store/gameStore";
import { classInfo } from "../../data/classes";
import type { ClassId, DamageType } from "../../types";

/**
 * D2-style cosmetic immunity hint per enemy archetype.
 * Server-authoritative resistances are not yet exposed per-enemy on the wire,
 * so this stays a static UI hint until the protocol grows that field.
 * Unknown ids fall back to no immunities.
 */
const KIND_IMMUNITIES: Record<string, DamageType[]> = {
  zombie: ["poison"],
  skeleton: ["poison", "cold"],
  fallen_one: [],
};

const KIND_LABEL: Record<string, string> = {
  zombie: "Zombie",
  skeleton: "Skeleton",
  fallen_one: "Fallen One",
};

const DAMAGE_LABEL: Record<DamageType, string> = {
  physical: "Physical",
  fire: "Fire",
  cold: "Cold",
  lightning: "Lightning",
  poison: "Poison",
  magical: "Magic",
};

/** Some classes have a flavor name for their secondary resource. */
function resourceLabel(classId: ClassId | null): string {
  switch (classId) {
    case "barbarian":
      return "Rage";
    case "necromancer":
    case "sorceress":
      return "Mana";
    default:
      return "Mana";
  }
}

function Bar({
  value,
  max,
  color,
  label,
}: {
  value: number;
  max: number;
  color: string;
  label: string;
}) {
  const pct = max > 0 ? Math.max(0, Math.min(1, value / max)) : 0;
  return (
    <div className="portrait-bar" title={`${label}: ${Math.floor(value)} / ${Math.floor(max)}`}>
      <div className="portrait-bar__fill" style={{ width: `${pct * 100}%`, background: color }} />
      <div className="portrait-bar__text">
        <span>{label}</span>
        <span>
          {Math.floor(value)} / {Math.floor(max)}
        </span>
      </div>
    </div>
  );
}

export function PlayerPortrait() {
  const playerName = useGameStore((s) => s.playerName);
  const level = useGameStore((s) => s.level);
  const hp = useGameStore((s) => s.hp);
  const maxHp = useGameStore((s) => s.maxHp);
  const mana = useGameStore((s) => s.mana);
  const maxMana = useGameStore((s) => s.maxMana);
  const xp = useGameStore((s) => s.xp);
  const xpToNext = useGameStore((s) => s.xpToNext);
  const classId = useGameStore((s) => s.classId);
  const connected = useGameStore((s) => s.connected);

  if (!connected) return null;

  const cls = classId ? classInfo(classId) : null;
  const icon = cls?.icon ?? "🧝";
  const name = playerName || "Hero";
  const xpPct =
    xpToNext > 0 ? Math.max(0, Math.min(1, xp / xpToNext)) : 0;

  return (
    <div className="portrait portrait--player">
      <div className="portrait__icon" aria-hidden>{icon}</div>
      <div className="portrait__body">
        <div className="portrait__name">
          {name}
          <span className="portrait__sub">Level {level}{cls ? ` · ${cls.name}` : ""}</span>
        </div>
        <Bar value={hp} max={maxHp} color="#a31515" label="Life" />
        <Bar value={mana} max={maxMana} color="#1d4ed8" label={resourceLabel(classId)} />
        <div
          className="portrait-bar portrait-bar--slim"
          title={`Experience: ${Math.floor(xp)} / ${Math.floor(xpToNext)}`}
        >
          <div
            className="portrait-bar__fill"
            style={{ width: `${xpPct * 100}%`, background: "#fbbf24" }}
          />
        </div>
      </div>
    </div>
  );
}

export function TargetPortrait() {
  const targetEnemyId = useGameStore((s) => s.targetEnemyId);
  const enemies = useGameStore((s) => s.enemies);

  if (!targetEnemyId) return null;
  const enemy = enemies.find((e) => e.id === targetEnemyId);
  if (!enemy || enemy.state === "dead") return null;

  const immunities = KIND_IMMUNITIES[enemy.kind] ?? [];

  return (
    <div className="portrait portrait--target">
      <div className="portrait__icon portrait__icon--enemy" aria-hidden>💀</div>
      <div className="portrait__body">
        <div className="portrait__name">
          {KIND_LABEL[enemy.kind] ?? enemy.kind}
          <span className="portrait__sub">Level {enemy.level}</span>
        </div>
        <Bar value={enemy.hp} max={enemy.maxHp} color="#a31515" label="Life" />
        {immunities.length > 0 ? (
          <div className="portrait__immune">
            Immun:{" "}
            {immunities.map((d, i) => (
              <span key={d} className="portrait__immune-tag">
                {DAMAGE_LABEL[d]}
                {i < immunities.length - 1 ? " · " : ""}
              </span>
            ))}
          </div>
        ) : (
          <div className="portrait__immune portrait__immune--none">No immunities</div>
        )}
      </div>
    </div>
  );
}

/** Combined portrait bar (top-left) — player always, target when selected. */
export default function PortraitBar() {
  return (
    <div className="portrait-bar-wrap">
      <PlayerPortrait />
      <TargetPortrait />
    </div>
  );
}
