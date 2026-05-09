import type { Item } from "../../types";
import { rarityColor } from "./rarity";

interface Props {
  item: Item;
}

/** D2-style hover tooltip: name (rarity color), slot/ilvl, affixes. */
export default function ItemTooltip({ item }: Props) {
  const color = rarityColor(item.rarity);
  return (
    <div className="item-tooltip">
      <div className="item-tooltip__name" style={{ color }}>
        {item.name}
      </div>
      <div className="item-tooltip__meta">
        {item.slot} · ilvl {item.itemLevel} · {item.rarity}
      </div>
      {item.affixes.length > 0 && (
        <ul className="item-tooltip__affixes">
          {item.affixes.map((a, i) => (
            <li key={i}>
              {a.label} {a.value > 0 ? `+${a.value}` : a.value}
            </li>
          ))}
        </ul>
      )}
      <div className="item-tooltip__value">Value: {item.vendorValue} G</div>
    </div>
  );
}
