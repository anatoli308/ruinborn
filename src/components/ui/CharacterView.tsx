import { useState } from "react";
import { useGameStore } from "../../store/gameStore";
import type { EquipSlotName, Item } from "../../types";
import ItemTooltip from "./ItemTooltip";
import { rarityColor } from "./rarity";

/** D2-style paperdoll: 10 equipment slots on a character silhouette. */
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
        title={item ? `${item.name} \u2014 click to unequip` : label}
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
            Attributes
            {unspentStatPoints > 0 && (
              <span className="char-attr-points"> · {unspentStatPoints} points free</span>
            )}
          </div>
          {(["strength", "dexterity", "vitality", "energy"] as const).map((stat) => {
            const labels: Record<typeof stat, string> = {
              strength: "Strength",
              dexterity: "Dexterity",
              vitality: "Vitality",
              energy: "Energy",
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
          <div className="char-attr-header">Resistances</div>
          {(
            [
              { key: "physical", label: "Physical", color: "#cbd5e1" },
              { key: "fire", label: "Fire", color: "#f97316" },
              { key: "cold", label: "Cold", color: "#60a5fa" },
              { key: "lightning", label: "Lightning", color: "#facc15" },
              { key: "poison", label: "Poison", color: "#84cc16" },
              { key: "magical", label: "Magic", color: "#c084fc" },
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
          {slot("helmet", "Helmet", "\u26D1\uFE0F", "helmet")}
          {slot("amulet", "Amulet", "\u{1F4FF}", "amulet")}
          {slot("weapon", "Weapon", "\u{1F5E1}\uFE0F", "weapon")}
          {slot("chest", "Chest", "\u{1F6E1}\uFE0F", "chest")}
          {slot("offhand", "Offhand", "\u{1F6E1}\uFE0F", "offhand")}
          {slot("gloves", "Gloves", "\u{1F9E4}", "gloves")}
          {slot("ring1", "Ring 1", "\u{1F48D}", "ring1")}
          {slot("belt", "Belt", "\u{1F45A}", "belt")}
          {slot("ring2", "Ring 2", "\u{1F48D}", "ring2")}
          {slot("boots", "Boots", "\u{1F45F}", "boots")}
        </div>

        <div className="char-view-hint">
          Click a slot to unequip {"\u00B7"} double-click an item from the inventory to equip
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
