import { useState } from "react";
import { useGameStore } from "../../store/gameStore";
import type { EquipSlotName, Item } from "../../types";
import ItemTooltip from "./ItemTooltip";
import { rarityColor } from "./rarity";

/** D2-style Paperdoll: 10 Equipment-Slots auf einer Charakter-Silhouette. */
export default function CharacterView() {
  const characterOpen = useGameStore((s) => s.characterOpen);
  const setCharacterOpen = useGameStore((s) => s.setCharacterOpen);
  const equipment = useGameStore((s) => s.equipment);
  const sendUnequipItem = useGameStore((s) => s.sendUnequipItem);
  const sendAllocateStat = useGameStore((s) => s.sendAllocateStat);
  const playerName = useGameStore((s) => s.playerName);
  const reputation = useGameStore((s) => s.reputation);
  const gold = useGameStore((s) => s.gold);
  const level = useGameStore((s) => s.level);
  const xp = useGameStore((s) => s.xp);
  const xpToNext = useGameStore((s) => s.xpToNext);
  const stats = useGameStore((s) => s.stats);
  const unspentStatPoints = useGameStore((s) => s.unspentStatPoints);
  const hp = useGameStore((s) => s.hp);
  const maxHp = useGameStore((s) => s.maxHp);
  const mana = useGameStore((s) => s.mana);
  const maxMana = useGameStore((s) => s.maxMana);
  const resistances = useGameStore((s) => s.resistances);

  const [hover, setHover] = useState<{ item: Item; x: number; y: number } | null>(null);

  if (!characterOpen) return null;

  const slot = (
    name: EquipSlotName,
    label: string,
    icon: string,
    gridArea: string,
  ) => {
    const item = equipment[name];
    return (
      <div
        key={name}
        className="char-slot"
        style={{
          gridArea,
          borderColor: item ? rarityColor(item.rarity) : "rgba(255,255,255,0.18)",
          boxShadow: item ? `0 0 12px ${rarityColor(item.rarity)}55` : "none",
          cursor: item ? "pointer" : "default",
        }}
        onClick={() => {
          if (item) sendUnequipItem(name);
        }}
        onMouseEnter={(e) => {
          if (item) setHover({ item, x: e.clientX, y: e.clientY });
        }}
        onMouseMove={(e) => {
          if (item) setHover({ item, x: e.clientX, y: e.clientY });
        }}
        onMouseLeave={() => setHover(null)}
        title={item ? `${item.name} \u2014 Klick zum Ablegen` : label}
      >
        {item ? (
          <span className="char-slot-icon">{item.icon}</span>
        ) : (
          <span className="char-slot-empty">{icon}</span>
        )}
        <span className="char-slot-label">{label}</span>
      </div>
    );
  };

  return (
    <>
      <div className="char-view">
        <div className="char-view-header">
          <span>{"\u{1F464} "}{playerName}</span>
          <button className="char-close" onClick={() => setCharacterOpen(false)}>
            {"\u00D7"}
          </button>
        </div>

        <div className="char-stats">
          <span>Lv {level} ({xp}/{xpToNext} XP)</span>
          <span style={{ color: "#e74c3c" }}>❤ {Math.floor(hp)}/{Math.floor(maxHp)}</span>
          <span style={{ color: "#3498db" }}>💧 {Math.floor(mana)}/{Math.floor(maxMana)}</span>
          <span>Gold: {Math.floor(gold)}</span>
          <span>Ruf: {reputation}</span>
        </div>

        <div className="char-attributes">
          <div className="char-attr-header">
            Attribute
            {unspentStatPoints > 0 && (
              <span className="char-attr-points"> · {unspentStatPoints} Punkte frei</span>
            )}
          </div>
          {(["strength", "dexterity", "vitality", "energy"] as const).map((stat) => {
            const labels: Record<typeof stat, string> = {
              strength: "Stärke",
              dexterity: "Geschick",
              vitality: "Vitalität",
              energy: "Energie",
            };
            return (
              <div key={stat} className="char-attr-row">
                <span className="char-attr-label">{labels[stat]}</span>
                <span className="char-attr-value">{stats[stat]}</span>
                <button
                  type="button"
                  className="char-attr-plus"
                  disabled={unspentStatPoints <= 0}
                  onClick={() => void sendAllocateStat(stat)}
                  title={`+1 ${labels[stat]}`}
                >
                  +
                </button>
              </div>
            );
          })}
        </div>

        <div className="char-resistances">
          <div className="char-attr-header">Widerstände</div>
          {(
            [
              { key: "physical", label: "Physisch", color: "#cbd5e1" },
              { key: "fire", label: "Feuer", color: "#f97316" },
              { key: "cold", label: "Kälte", color: "#60a5fa" },
              { key: "lightning", label: "Blitz", color: "#facc15" },
              { key: "poison", label: "Gift", color: "#84cc16" },
              { key: "magical", label: "Magisch", color: "#c084fc" },
            ] as const
          ).map((r) => (
            <div key={r.key} className="char-attr-row">
              <span className="char-attr-label" style={{ color: r.color }}>
                {r.label}
              </span>
              <span className="char-attr-value">
                {resistances[r.key]}%
              </span>
            </div>
          ))}
        </div>

        <div className="char-paperdoll">
          {slot("helmet", "Helm", "\u26D1\uFE0F", "helmet")}
          {slot("amulet", "Amulett", "\u{1F4FF}", "amulet")}
          {slot("weapon", "Waffe", "\u{1F5E1}\uFE0F", "weapon")}
          {slot("chest", "Brust", "\u{1F6E1}\uFE0F", "chest")}
          {slot("offhand", "Offhand", "\u{1F6E1}\uFE0F", "offhand")}
          {slot("gloves", "Handschuhe", "\u{1F9E4}", "gloves")}
          {slot("ring1", "Ring 1", "\u{1F48D}", "ring1")}
          {slot("belt", "G\u00FCrtel", "\u{1F45A}", "belt")}
          {slot("ring2", "Ring 2", "\u{1F48D}", "ring2")}
          {slot("boots", "Stiefel", "\u{1F45F}", "boots")}
        </div>

        <div className="char-view-hint">
          Klick auf einen Slot zum Ablegen {"\u00B7"} Items aus dem Inventar mit Doppelklick anlegen
        </div>
      </div>

      {hover && (
        <div
          style={{
            position: "fixed",
            left: hover.x + 16,
            top: hover.y + 16,
            zIndex: 100,
            pointerEvents: "none",
          }}
        >
          <ItemTooltip item={hover.item} />
        </div>
      )}
    </>
  );
}
