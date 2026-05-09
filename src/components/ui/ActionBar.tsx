import { useGameStore } from "../../store/gameStore";
import { rarityColor } from "./rarity";
import { SKILL_CATALOG } from "../../data/classes";
import { DEFAULT_ATTACK_ICON } from "../../assets/spell_icons";
import SkillIconView from "./SkillIconView";
import type { ActionBinding, SkillEffectKind } from "../../types";

/** Tiny emoji icon per skill effect — kept here so the action bar stays self-contained. */
const SKILL_ICON: Record<SkillEffectKind, string> = {
  direct_damage: "⚔️",
  aoe_around: "💥",
  damage_over_time: "🔥",
  teleport: "✨",
  self_buff: "🛡️",
  placeholder: "❔",
};

interface BindingVisual {
  icon: string;
  title: string;
  borderColor?: string;
}

function describeBinding(
  binding: ActionBinding | null,
  itemIndex: Map<string, { icon: string; name: string; rarity: string }>,
  fallback: string,
  fallbackTitle: string,
): BindingVisual {
  if (!binding) return { icon: fallback, title: fallbackTitle };
  if (binding.kind === "item") {
    const item = itemIndex.get(binding.itemId);
    if (item) {
      return {
        icon: item.icon,
        title: item.name,
        borderColor: rarityColor(item.rarity as never),
      };
    }
    return { icon: "❓", title: "Unbekanntes Item" };
  }
  if (binding.kind === "skill") {
    const skill = SKILL_CATALOG.find((s) => s.id === binding.skillId);
    return {
      icon: skill?.icon ?? (skill ? SKILL_ICON[skill.effect] : "❔"),
      title: skill?.name ?? "Skill",
    };
  }
  if (binding.kind === "attack") {
    return { icon: DEFAULT_ATTACK_ICON, title: "Standardangriff" };
  }
  return { icon: fallback, title: fallbackTitle };
}

/** Diablo 2-style: HP-orb · LMB · 1..9 · RMB · Mana-orb. */
export default function ActionBar() {
  const actionBar = useGameStore((s) => s.actionBar);
  const bags = useGameStore((s) => s.bags);
  const sendUseActionSlot = useGameStore((s) => s.sendUseActionSlot);
  const sendSetActionSlotSkill = useGameStore((s) => s.sendSetActionSlotSkill);
  const sendBindMouseSkill = useGameStore((s) => s.sendBindMouseSkill);
  const setHoveredSkillRange = useGameStore((s) => s.setHoveredSkillRange);
  const mouseLeft = useGameStore((s) => s.mouseLeft);
  const mouseRight = useGameStore((s) => s.mouseRight);
  const hp = useGameStore((s) => s.hp);
  const maxHp = useGameStore((s) => s.maxHp);
  const mana = useGameStore((s) => s.mana);
  const maxMana = useGameStore((s) => s.maxMana);

  /** Resolve the skill range hidden behind a binding, or `null` if there is none. */
  const bindingRange = (binding: ActionBinding | null): number | null => {
    if (!binding || binding.kind !== "skill") return null;
    const skill = SKILL_CATALOG.find((s) => s.id === binding.skillId);
    return skill && skill.range > 0 ? skill.range : null;
  };

  const showRangeFor = (binding: ActionBinding | null) => {
    setHoveredSkillRange(bindingRange(binding));
  };
  const clearRange = () => setHoveredSkillRange(null);

  const itemIndex = new Map<string, { icon: string; name: string; rarity: string }>();
  for (const bag of bags.bags) {
    if (!bag) continue;
    for (const slot of bag.slots) {
      if (slot) itemIndex.set(slot.id, { icon: slot.icon, name: slot.name, rarity: slot.rarity });
    }
  }

  const lmb = describeBinding(mouseLeft, itemIndex, DEFAULT_ATTACK_ICON, "Linksklick (Standardangriff)");
  const rmb = describeBinding(mouseRight, itemIndex, "✋", "Rechtsklick (leer)");

  const hpPct = maxHp > 0 ? Math.max(0, Math.min(1, hp / maxHp)) : 0;
  const manaPct = maxMana > 0 ? Math.max(0, Math.min(1, mana / maxMana)) : 0;

  /** Read a skill drop payload, if any. Future-proof: returns null when not a skill. */
  const readSkillDrop = (e: React.DragEvent): string | null => {
    const raw = e.dataTransfer.getData("application/x-ruinborn-skill");
    if (!raw) return null;
    try {
      const parsed = JSON.parse(raw) as { skillId?: string };
      return parsed.skillId ?? null;
    } catch {
      return null;
    }
  };

  const allowDrop = (e: React.DragEvent) => {
    if (e.dataTransfer.types.includes("application/x-ruinborn-skill")) {
      e.preventDefault();
      e.dataTransfer.dropEffect = "copy";
    }
  };

  return (
    <div className="action-bar">
      <Orb kind="hp" value={Math.floor(hp)} max={Math.floor(maxHp)} pct={hpPct} />

      <button
        type="button"
        className="action-bar__slot action-bar__mouse"
        title={`Linksklick: ${lmb.title}`}
        style={lmb.borderColor ? { borderColor: lmb.borderColor } : undefined}
        onMouseEnter={() => showRangeFor(mouseLeft)}
        onMouseLeave={clearRange}
        onDragOver={allowDrop}
        onDrop={(e) => {
          const skillId = readSkillDrop(e);
          if (!skillId) return;
          e.preventDefault();
          void sendBindMouseSkill(0, skillId);
        }}
      >
        <span className="action-bar__hotkey">LMB</span>
        <SkillIconView icon={lmb.icon} className="action-bar__icon" alt={lmb.title} />
      </button>

      {actionBar.slots.map((binding, i) => {
        const v = describeBinding(binding, itemIndex, "", "");
        return (
          <button
            key={i}
            type="button"
            className="action-bar__slot"
            title={v.title}
            onClick={() => void sendUseActionSlot(i)}
            onMouseEnter={() => showRangeFor(binding)}
            onMouseLeave={clearRange}
            onDragOver={allowDrop}
            onDrop={(e) => {
              const skillId = readSkillDrop(e);
              if (!skillId) return;
              e.preventDefault();
              void sendSetActionSlotSkill(i, skillId);
            }}
            style={v.borderColor ? { borderColor: v.borderColor } : undefined}
          >
            <span className="action-bar__hotkey">{i + 1}</span>
            {v.icon ? (
              <SkillIconView icon={v.icon} className="action-bar__icon" alt={v.title} />
            ) : null}
          </button>
        );
      })}

      <button
        type="button"
        className="action-bar__slot action-bar__mouse"
        title={`Rechtsklick: ${rmb.title}`}
        style={rmb.borderColor ? { borderColor: rmb.borderColor } : undefined}
        onMouseEnter={() => showRangeFor(mouseRight)}
        onMouseLeave={clearRange}
        onDragOver={allowDrop}
        onDrop={(e) => {
          const skillId = readSkillDrop(e);
          if (!skillId) return;
          e.preventDefault();
          void sendBindMouseSkill(1, skillId);
        }}
      >
        <span className="action-bar__hotkey">RMB</span>
        <SkillIconView icon={rmb.icon} className="action-bar__icon" alt={rmb.title} />
      </button>

      <Orb kind="mana" value={Math.floor(mana)} max={Math.floor(maxMana)} pct={manaPct} />
    </div>
  );
}

function Orb({
  kind,
  value,
  max,
  pct,
}: {
  kind: "hp" | "mana";
  value: number;
  max: number;
  pct: number;
}) {
  const fillColor = kind === "hp" ? "#a31515" : "#1d4ed8";
  const glow = kind === "hp" ? "#ff5050" : "#5b8dff";
  return (
    <div className={`d2-orb d2-orb--${kind}`} title={`${value} / ${max}`}>
      <div
        className="d2-orb__fill"
        style={{
          height: `${pct * 100}%`,
          background: `linear-gradient(180deg, ${glow} 0%, ${fillColor} 100%)`,
        }}
      />
      <div className="d2-orb__rim" />
      <div className="d2-orb__label">
        {value}
        <span className="d2-orb__sep">/</span>
        {max}
      </div>
    </div>
  );
}

