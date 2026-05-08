import type { Rarity } from "../../types";

/** D2-style rarity color mapping. */
export function rarityColor(rarity: Rarity): string {
  switch (rarity) {
    case "Common":
      return "#9d9d9d";
    case "Magic":
      return "#4d80ff";
    case "Rare":
      return "#ffd700";
    case "Epic":
      return "#a335ee";
    case "Legendary":
      return "#ff8000";
  }
}
