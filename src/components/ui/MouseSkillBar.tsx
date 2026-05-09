import { useGameStore } from "../../store/gameStore";
import type { ActionBinding } from "../../types";

/** Two slots for left/right mouse skills (D2-style). Click to clear → Attack default. */
export default function MouseSkillBar() {
  const mouseLeft = useGameStore((s) => s.mouseLeft);
  const mouseRight = useGameStore((s) => s.mouseRight);
  const bags = useGameStore((s) => s.bags);
  const sendSetMouseSkill = useGameStore((s) => s.sendSetMouseSkill);

  const itemIndex = new Map<string, { icon: string; name: string }>();
  for (const bag of bags.bags) {
    if (!bag) continue;
    for (const slot of bag.slots) {
      if (slot) itemIndex.set(slot.id, { icon: slot.icon, name: slot.name });
    }
  }

  const renderSlot = (binding: ActionBinding | null, button: 0 | 1, label: string) => {
    let icon = "⚔";
    let title = "Attack (default)";
    if (binding && binding.kind === "item") {
      const item = itemIndex.get(binding.itemId);
      icon = item?.icon ?? "?";
      title = item?.name ?? binding.itemId;
    }
    return (
      <button
        key={button}
        type="button"
        className="mouse-skill-slot"
        title={`${label}: ${title} — click to reset to attack`}
        onClick={() => void sendSetMouseSkill(button, null)}
      >
        <span className="mouse-skill-label">{label}</span>
        <span className="mouse-skill-icon">{icon}</span>
      </button>
    );
  };

  return (
    <div className="mouse-skill-bar">
      {renderSlot(mouseLeft, 0, "LMB")}
      {renderSlot(mouseRight, 1, "RMB")}
    </div>
  );
}
