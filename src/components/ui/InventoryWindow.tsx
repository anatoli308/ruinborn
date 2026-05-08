import { useEffect, useState } from "react";
import { useGameStore } from "../../store/gameStore";
import type { Item } from "../../types";
import ItemTooltip from "./ItemTooltip";
import { rarityColor } from "./rarity";

interface CursorItem {
  bag: number;
  slot: number;
  item: Item;
}

/** D2/WoW-style Inventar-Fenster mit Pickup-Cursor und Right-Click-Bind. */
export default function InventoryWindow() {
  const inventoryOpen = useGameStore((s) => s.inventoryOpen);
  const setInventoryOpen = useGameStore((s) => s.setInventoryOpen);
  const bags = useGameStore((s) => s.bags);
  const actionBar = useGameStore((s) => s.actionBar);
  const sendMoveItem = useGameStore((s) => s.sendMoveItem);
  const sendDropItem = useGameStore((s) => s.sendDropItem);
  const sendSetActionSlot = useGameStore((s) => s.sendSetActionSlot);
  const sendEquipItem = useGameStore((s) => s.sendEquipItem);

  const [cursor, setCursor] = useState<CursorItem | null>(null);
  const [hover, setHover] = useState<{ item: Item; x: number; y: number } | null>(null);
  const [mouse, setMouse] = useState({ x: 0, y: 0 });

  useEffect(() => {
    if (!cursor) return;
    const onMove = (e: MouseEvent) => setMouse({ x: e.clientX, y: e.clientY });
    window.addEventListener("mousemove", onMove);
    return () => window.removeEventListener("mousemove", onMove);
  }, [cursor]);

  if (!inventoryOpen) return null;

  // Welche Bag wir anzeigen — MVP: nur Bag 0 (4×4 Grid).
  const bag = bags.bags[0];

  function handleSlotClick(bagIdx: number, slotIdx: number) {
    const target = bags.bags[bagIdx]?.slots[slotIdx] ?? null;

    if (cursor) {
      // Place / swap.
      void sendMoveItem(cursor.bag, cursor.slot, bagIdx, slotIdx);
      setCursor(null);
    } else if (target) {
      // Pickup.
      setCursor({ bag: bagIdx, slot: slotIdx, item: target });
    }
  }

  function handleRightClick(e: React.MouseEvent, item: Item) {
    e.preventDefault();
    // Auto-bind to first empty action-slot.
    const emptyIdx = actionBar.slots.findIndex((s) => s === null);
    if (emptyIdx >= 0) {
      void sendSetActionSlot(emptyIdx, item.id);
    }
  }

  function handleDoubleClick(bagIdx: number, slotIdx: number, item: Item | null) {
    if (!item) return;
    if (item.slot === "Bag") return;
    void sendEquipItem(bagIdx, slotIdx);
    if (cursor) setCursor(null);
  }

  return (
    <div className="inv-window">
      <header className="inv-window__header">
        <h2>Inventar</h2>
        <button type="button" onClick={() => setInventoryOpen(false)} className="inv-window__close">
          ✕
        </button>
      </header>

      <div className="inv-window__bag-tabs">
        {bags.bags.map((b, i) => (
          <span key={i} className={`inv-window__tab${b ? " inv-window__tab--filled" : ""}`}>
            {b ? `🎒 ${b.name}` : "—"}
          </span>
        ))}
      </div>

      {bag ? (
        <div className="inv-grid" style={{ gridTemplateColumns: "repeat(4, 1fr)" }}>
          {bag.slots.map((item, slotIdx) => (
            <button
              key={slotIdx}
              type="button"
              className={`inv-cell${item ? " inv-cell--filled" : ""}`}
              style={item ? { borderColor: rarityColor(item.rarity) } : undefined}
              onClick={() => handleSlotClick(0, slotIdx)}
              onDoubleClick={() => handleDoubleClick(0, slotIdx, item)}
              onContextMenu={(e) => item && handleRightClick(e, item)}
              onMouseEnter={(e) =>
                item && setHover({ item, x: e.clientX, y: e.clientY })
              }
              onMouseMove={(e) =>
                item && setHover({ item, x: e.clientX, y: e.clientY })
              }
              onMouseLeave={() => setHover(null)}
            >
              {item ? (
                <span className="inv-cell__icon" style={{ color: rarityColor(item.rarity) }}>
                  {item.icon}
                </span>
              ) : null}
            </button>
          ))}
        </div>
      ) : (
        <p className="inv-window__empty">Kein aktiver Beutel.</p>
      )}

      <footer className="inv-window__footer">
        <small>Linksklick: Aufnehmen / Ablegen · Doppelklick: Anlegen · Rechtsklick: Auf Action-Bar legen</small>
        {cursor && (
          <button
            type="button"
            className="inv-window__drop"
            onClick={() => {
              void sendDropItem(cursor.bag, cursor.slot);
              setCursor(null);
            }}
          >
            🗑️ [{cursor.item.name}] wegwerfen
          </button>
        )}
      </footer>

      {/* Pickup-Cursor — folgt der Maus */}
      {cursor && (
        <div
          className="inv-cursor"
          style={{
            color: rarityColor(cursor.item.rarity),
            left: mouse.x,
            top: mouse.y,
          }}
        >
          {cursor.item.icon}
        </div>
      )}

      {/* Hover-Tooltip */}
      {hover && (
        <div
          className="inv-tooltip-anchor"
          style={{ left: hover.x + 16, top: hover.y + 16 }}
        >
          <ItemTooltip item={hover.item} />
        </div>
      )}
    </div>
  );
}
