export type CommodityCategory = "food" | "material" | "luxury" | "military" | "technology";

export interface Commodity {
  id: string;
  name: string;
  icon: string;
  category: CommodityCategory;
  baseValue: number;
}

export interface PlayerMarket {
  id: string;
  ownerId: string;
  ownerName: string;
  name: string;
  x: number;
  z: number;
  orders: MarketOrder[];
}

export interface MarketOrder {
  id: string;
  commodityId: string;
  orderType: "buy" | "sell";
  quantity: number;
  remaining: number;
  pricePerUnit: number;
  createdTick: number;
}

// ─── D2 World / Zones ────────────────────────────────────────────────────

export type ZoneId = "town" | "wilderness" | "burial_grounds";
export type ZoneKind = "town" | "wilderness" | "dungeon";

export interface ZoneBounds {
  minX: number;
  maxX: number;
  minZ: number;
  maxZ: number;
}

export interface Zone {
  id: ZoneId;
  name: string;
  kind: ZoneKind;
  bounds: ZoneBounds;
  spawnX: number;
  spawnZ: number;
  waypointX: number | null;
  waypointZ: number | null;
  enemyTarget: number;
}

// ─── D2 Combat / Enemies / Loot ──────────────────────────────────────────

// Enemy archetype id from server JSON (e.g. "zombie", "skeleton", "fallen_one").
// Adding a new monster on the server only requires editing
// `crates/ruinborn-game/data/enemies.json` — no TS changes needed.
export type EnemyKind = string;
export type EnemyState = "idle" | "chase" | "attack" | "dead";

export interface Enemy {
  id: string;
  kind: EnemyKind;
  zone: ZoneId;
  x: number;
  z: number;
  hp: number;
  maxHp: number;
  damage: number;
  level: number;
  moveSpeed: number;
  xpReward: number;
  state: EnemyState;
  targetPlayerId: string | null;
  attackCooldown: number;
  despawnIn: number;
  spawnX: number;
  spawnZ: number;
}

export interface LootDrop {
  id: string;
  item: Item;
  x: number;
  z: number;
  zone: ZoneId;
  droppedTick: number;
}

// ─── D2 Progression ──────────────────────────────────────────────────────

export interface Stats {
  strength: number;
  dexterity: number;
  vitality: number;
  energy: number;
}

export interface Mission {
  id: string;
  title: string;
  description: string;
  missionType: "gather" | "sell";
  commodityId: string | null;
  targetQuantity: number;
  progress: number;
  rewardGold: number;
  rewardItems: Record<string, number>;
  rewardReputation: number;
  expiresTick: number;
}

export interface TradeRecord {
  commodityId: string;
  tradeType: "buy" | "sell";
  quantity: number;
  pricePerUnit: number;
  marketId: string;
  tick: number;
}

export interface OtherPlayer {
  id: string;
  name: string;
  x: number;
  z: number;
}

// ─── Items / Bags / ActionBar (Diablo-style random loot) ─────────────────

export type Rarity = "Common" | "Magic" | "Rare" | "Epic" | "Legendary";

export type ItemSlot =
  | "Weapon"
  | "Offhand"
  | "Helmet"
  | "Chest"
  | "Belt"
  | "Boots"
  | "Gloves"
  | "Ring"
  | "Amulet"
  | "Bag";

export interface Affix {
  stat: string;
  value: number;
  label: string;
  position: "Prefix" | "Suffix";
}

export interface Item {
  id: string;
  name: string;
  baseName: string;
  icon: string;
  slot: ItemSlot;
  rarity: Rarity;
  itemLevel: number;
  affixes: Affix[];
  vendorValue: number;
}

export interface Bag {
  name: string;
  fixed: boolean;
  slots: Array<Item | null>;
}

export interface ItemBags {
  bags: Array<Bag | null>;
}

export type ActionBinding =
  | { kind: "item"; itemId: string }
  | { kind: "attack" }
  | { kind: "skill"; skillId: string };

export interface ActionBar {
  slots: Array<ActionBinding | null>;
}

// ─── Equipment (D2-style Paperdoll) ──────────────────────────────────────

export type EquipSlotName =
  | "helmet"
  | "amulet"
  | "chest"
  | "belt"
  | "gloves"
  | "boots"
  | "weapon"
  | "offhand"
  | "ring1"
  | "ring2";

export interface Equipment {
  helmet: Item | null;
  amulet: Item | null;
  chest: Item | null;
  belt: Item | null;
  gloves: Item | null;
  boots: Item | null;
  weapon: Item | null;
  offhand: Item | null;
  ring1: Item | null;
  ring2: Item | null;
}

// ─── D2 Classes / Skills ─────────────────────────────────────────────────

export type ClassId = "barbarian" | "sorceress" | "necromancer";

export interface ClassInfo {
  id: ClassId;
  name: string;
  tagline: string;
  icon: string;
  baseStats: Stats;
  starterSkills: string[];
}

export type SkillEffectKind =
  | "direct_damage"
  | "aoe_around"
  | "damage_over_time"
  | "teleport"
  | "self_buff"
  | "placeholder";

export type DamageType =
  | "physical"
  | "fire"
  | "cold"
  | "lightning"
  | "poison"
  | "magical";

export type DamageTag =
  | "melee"
  | "ranged"
  | "spell"
  | "summoning"
  | "trap";

export interface Resistances {
  physical: number;
  fire: number;
  cold: number;
  lightning: number;
  poison: number;
  magical: number;
}

export interface DotInstance {
  damage_type: DamageType;
  damage_per_tick: number;
  ticks_remaining: number;
  tags: DamageTag[];
}

export interface SkillDef {
  id: string;
  name: string;
  classId: ClassId;
  manaCost: number;
  cooldownTicks: number;
  range: number;
  requiresLevel: number;
  effect: SkillEffectKind;
  damageType: DamageType | null;
  tags: DamageTag[];
  description: string;
  /** Cosmetic UI icon. Falls back to a generic per-effect icon when omitted. */
  icon?: string;
}
