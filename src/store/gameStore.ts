import { create } from "zustand";
import { wsTransport } from "../services/wsTransport";
import type {
  ActionBar,
  ActionBinding,
  Affix,
  Bag,
  ClassId,
  Commodity,
  DotInstance,
  Enemy,
  EnemyKind,
  EnemyState,
  EquipSlotName,
  Equipment,
  Item,
  ItemBags,
  ItemSlot,
  LootDrop,
  MarketOrder,
  Mission,
  OtherPlayer,
  PlayerMarket,
  Rarity,
  Resistances,
  Stats,
  TradeRecord,
  Zone,
  ZoneId,
  ZoneKind,
} from "../types";

// ── Server Snapshot Shape (mirrors Rust PlayerSnapshot) ─────

interface ServerPlayerSnapshot {
  tick: number;
  elapsed_secs: number;
  player: ServerPlayer;
  other_players: ServerOtherPlayer[];
  commodities: ServerCommodity[];
  player_markets: ServerPlayerMarket[];
  mission_board: ServerMission[];
  zones: ServerZone[];
  enemies: ServerEnemy[];
  loot_drops: ServerLootDrop[];
}

interface ServerStats {
  strength: number;
  dexterity: number;
  vitality: number;
  energy: number;
}

interface ServerPlayer {
  id: string;
  name: string;
  x: number;
  z: number;
  gold: number;
  inventory: Record<string, number>;
  reputation: number;
  active_missions: ServerMission[];
  owned_market_id: string | null;
  nearest_market_id: string | null;
  show_trade_panel: boolean;
  trade_history: ServerTradeRecord[];
  notification: string;
  bags: ServerItemBags;
  action_bar: ServerActionBar;
  equipment: ServerEquipment;
  level: number;
  xp: number;
  xp_to_next: number;
  unspent_stat_points: number;
  stats: ServerStats;
  hp: number;
  max_hp: number;
  mana: number;
  max_mana: number;
  is_dead: boolean;
  respawn_in: number;
  zone: string;
  unlocked_waypoints: string[];
  mouse_left: ServerActionBinding | null;
  mouse_right: ServerActionBinding | null;
  class_id: ClassId | null;
  allocated_skills: Record<string, number>;
  unspent_skill_points: number;
  skill_cooldowns: Record<string, number>;
  active_buffs: Record<string, number>;
  resistances?: Resistances;
  dots?: DotInstance[];
}

interface ServerAffix {
  stat: string;
  value: number;
  label: string;
  position: "Prefix" | "Suffix";
}

interface ServerItem {
  id: string;
  name: string;
  base_name: string;
  icon: string;
  slot: string;
  rarity: string;
  item_level: number;
  affixes: ServerAffix[];
  vendor_value: number;
}

interface ServerBag {
  name: string;
  fixed: boolean;
  slots: Array<ServerItem | null>;
}

interface ServerItemBags {
  bags: Array<ServerBag | null>;
}

type ServerActionBinding =
  | { kind: "item"; item_id: string }
  | { kind: "attack" }
  | { kind: "skill"; skill_id: string };

interface ServerActionBar {
  slots: Array<ServerActionBinding | null>;
}

interface ServerEquipment {
  helmet: ServerItem | null;
  amulet: ServerItem | null;
  chest: ServerItem | null;
  belt: ServerItem | null;
  gloves: ServerItem | null;
  boots: ServerItem | null;
  weapon: ServerItem | null;
  offhand: ServerItem | null;
  ring1: ServerItem | null;
  ring2: ServerItem | null;
}

interface ServerOtherPlayer {
  id: string;
  name: string;
  x: number;
  z: number;
}

interface ServerCommodity {
  id: string;
  name: string;
  icon: string;
  category: string;
  base_value: number;
}

interface ServerPlayerMarket {
  id: string;
  owner_id: string;
  owner_name: string;
  name: string;
  x: number;
  z: number;
  orders: ServerMarketOrder[];
}

interface ServerMarketOrder {
  id: string;
  commodity_id: string;
  order_type: string;
  quantity: number;
  remaining: number;
  price_per_unit: number;
  created_tick: number;
}

interface ServerZoneBounds {
  min_x: number;
  max_x: number;
  min_z: number;
  max_z: number;
}

interface ServerZone {
  id: string;
  name: string;
  kind: string;
  bounds: ServerZoneBounds;
  spawn_x: number;
  spawn_z: number;
  waypoint_x: number | null;
  waypoint_z: number | null;
  enemy_target: number;
}

interface ServerEnemy {
  id: string;
  kind: string;
  zone: string;
  x: number;
  z: number;
  hp: number;
  max_hp: number;
  damage: number;
  level: number;
  move_speed: number;
  xp_reward: number;
  state: string;
  target_player_id: string | null;
  attack_cooldown: number;
  despawn_in: number;
  spawn_x: number;
  spawn_z: number;
}

interface ServerLootDrop {
  id: string;
  item: ServerItem;
  x: number;
  z: number;
  zone: string;
  dropped_tick: number;
}

interface ServerMission {
  id: string;
  title: string;
  description: string;
  mission_type: string;
  commodity_id: string | null;
  target_quantity: number;
  progress: number;
  reward_gold: number;
  reward_items: Record<string, number>;
  reward_reputation: number;
  expires_tick: number;
}

interface ServerTradeRecord {
  commodity_id: string;
  trade_type: string;
  quantity: number;
  price_per_unit: number;
  market_id: string;
  tick: number;
}

interface ServerMessage {
  type: string;
  snapshot?: ServerPlayerSnapshot | ServerDeltaSnapshot;
  success?: boolean;
  message?: string;
  player_id?: string;
}

interface ServerDeltaSnapshot {
  tick: number;
  elapsed_secs: number;
  player: ServerPlayer;
  other_players: ServerOtherPlayer[];
  player_markets?: ServerPlayerMarket[];
  mission_board?: ServerMission[];
  enemies: ServerEnemy[];
  loot_drops: ServerLootDrop[];
}

// ── Frontend Store (thin client) ─────────────────────────────

interface GameStore {
  // State (read-only mirror of server)
  tick: number;
  elapsedSecs: number;
  playerId: string;
  playerName: string;
  playerX: number;
  playerZ: number;
  gold: number;
  inventory: Record<string, number>;
  reputation: number;
  activeMissions: Mission[];
  ownedMarketId: string | null;
  nearestMarketId: string | null;
  commodities: Commodity[];
  playerMarkets: PlayerMarket[];
  missionBoard: Mission[];
  tradeHistory: TradeRecord[];
  otherPlayers: OtherPlayer[];
  showTradePanel: boolean;
  notification: string;
  connected: boolean;
  joining: boolean;
  bags: ItemBags;
  actionBar: ActionBar;
  equipment: Equipment;
  inventoryOpen: boolean;
  characterOpen: boolean;

  // D2 progression
  level: number;
  xp: number;
  xpToNext: number;
  unspentStatPoints: number;
  stats: Stats;
  hp: number;
  maxHp: number;
  mana: number;
  maxMana: number;
  isDead: boolean;
  respawnIn: number;

  // Zones / Combat
  zone: ZoneId;
  unlockedWaypoints: ZoneId[];
  zones: Zone[];
  enemies: Enemy[];
  lootDrops: LootDrop[];
  mouseLeft: ActionBinding | null;
  mouseRight: ActionBinding | null;

  // Class & skills
  classId: ClassId | null;
  allocatedSkills: Record<string, number>;
  unspentSkillPoints: number;
  skillCooldowns: Record<string, number>;
  activeBuffs: Record<string, number>;
  skillTreeOpen: boolean;
  waypointMenuOpen: boolean;
  /** Currently focused enemy (Tab/click). Cleared when the enemy dies or despawns. */
  targetEnemyId: string | null;

  /** Transient error toast (e.g. "Ziel außer Reichweite"). Cleared automatically by the Toast component. */
  lastError: { message: string; ts: number } | null;
  /** Range (world units) of the skill currently hovered in the action bar. `null` = hide indicator. */
  hoveredSkillRange: number | null;
  setHoveredSkillRange: (range: number | null) => void;
  clearLastError: () => void;

  // Damage model (Phase 3)
  resistances: Resistances;
  dots: DotInstance[];

  // Actions (send commands via WebSocket)
  sendMove: (dx: number, dz: number) => void;
  sendCreateMarket: (name: string) => Promise<{ success: boolean; message: string }>;
  sendPostOrder: (commodityId: string, orderType: "buy" | "sell", quantity: number, pricePerUnit: number) => Promise<{ success: boolean; message: string }>;
  sendCancelOrder: (orderId: string) => Promise<{ success: boolean; message: string }>;
  sendFillOrder: (marketId: string, orderId: string, quantity: number) => Promise<{ success: boolean; message: string }>;
  sendAcceptMission: (missionId: string) => Promise<{ success: boolean; message: string }>;
  sendToggleTradePanel: () => void;
  sendCloseTradePanel: () => void;
  sendMoveItem: (srcBag: number, srcSlot: number, dstBag: number, dstSlot: number) => Promise<{ success: boolean; message: string }>;
  sendDropItem: (bag: number, slot: number) => Promise<{ success: boolean; message: string }>;
  sendSetActionSlot: (slot: number, itemId: string | null) => Promise<{ success: boolean; message: string }>;
  sendSetActionSlotSkill: (slot: number, skillId: string) => Promise<{ success: boolean; message: string }>;
  sendUseActionSlot: (slot: number) => Promise<{ success: boolean; message: string }>;
  sendEquipItem: (bag: number, slot: number, target?: EquipSlotName | null) => Promise<{ success: boolean; message: string }>;
  sendUnequipItem: (target: EquipSlotName) => Promise<{ success: boolean; message: string }>;
  sendAttack: (enemyId: string, mouseButton: 0 | 1) => Promise<{ success: boolean; message: string }>;
  sendPickupLoot: (lootId: string) => Promise<{ success: boolean; message: string }>;
  sendTravelWaypoint: (zone: ZoneId) => Promise<{ success: boolean; message: string }>;
  sendAllocateStat: (stat: "strength" | "dexterity" | "vitality" | "energy") => Promise<{ success: boolean; message: string }>;
  sendSetMouseSkill: (mouseButton: 0 | 1, itemId: string | null) => Promise<{ success: boolean; message: string }>;
  sendBindMouseSkill: (mouseButton: 0 | 1, skillId: string | null) => Promise<{ success: boolean; message: string }>;
  sendChooseClass: (classId: ClassId) => Promise<{ success: boolean; message: string }>;
  sendAllocateSkill: (skillId: string) => Promise<{ success: boolean; message: string }>;
  sendCastSkill: (
    skillId: string,
    targetEnemyId: string | null,
    targetX: number | null,
    targetZ: number | null,
  ) => Promise<{ success: boolean; message: string }>;
  toggleInventory: () => void;
  setInventoryOpen: (open: boolean) => void;
  toggleCharacter: () => void;
  setCharacterOpen: (open: boolean) => void;
  toggleSkillTree: () => void;
  setSkillTreeOpen: (open: boolean) => void;
  toggleWaypointMenu: () => void;
  setWaypointMenuOpen: (open: boolean) => void;
  setTargetEnemy: (id: string | null) => void;
  cycleTarget: () => void;
  initConnection: (playerName: string, serverUrl?: string) => void;
}

// ── Mapping Helpers ──────────────────────────────────────────

function mapCommodity(c: ServerCommodity): Commodity {
  return { id: c.id, name: c.name, icon: c.icon, category: c.category as Commodity["category"], baseValue: c.base_value };
}

function mapMarket(m: ServerPlayerMarket): PlayerMarket {
  return {
    id: m.id,
    ownerId: m.owner_id,
    ownerName: m.owner_name,
    name: m.name,
    x: m.x,
    z: m.z,
    orders: m.orders.map(mapOrder),
  };
}

function mapOrder(o: ServerMarketOrder): MarketOrder {
  return {
    id: o.id,
    commodityId: o.commodity_id,
    orderType: o.order_type as "buy" | "sell",
    quantity: o.quantity,
    remaining: o.remaining,
    pricePerUnit: o.price_per_unit,
    createdTick: o.created_tick,
  };
}

function mapMission(m: ServerMission): Mission {
  return {
    id: m.id,
    title: m.title,
    description: m.description,
    missionType: m.mission_type as "gather" | "sell",
    commodityId: m.commodity_id,
    targetQuantity: m.target_quantity,
    progress: m.progress,
    rewardGold: m.reward_gold,
    rewardItems: m.reward_items,
    rewardReputation: m.reward_reputation,
    expiresTick: m.expires_tick,
  };
}

function mapTradeRecord(t: ServerTradeRecord): TradeRecord {
  return {
    commodityId: t.commodity_id,
    tradeType: t.trade_type as "buy" | "sell",
    quantity: t.quantity,
    pricePerUnit: t.price_per_unit,
    marketId: t.market_id,
    tick: t.tick,
  };
}

function mapOtherPlayer(p: ServerOtherPlayer): OtherPlayer {
  return { id: p.id, name: p.name, x: p.x, z: p.z };
}

function mapAffix(a: ServerAffix): Affix {
  return { stat: a.stat, value: a.value, label: a.label, position: a.position };
}

function mapItem(i: ServerItem): Item {
  return {
    id: i.id,
    name: i.name,
    baseName: i.base_name,
    icon: i.icon,
    slot: i.slot as ItemSlot,
    rarity: i.rarity as Rarity,
    itemLevel: i.item_level,
    affixes: i.affixes.map(mapAffix),
    vendorValue: i.vendor_value,
  };
}

function mapBag(b: ServerBag | null): Bag | null {
  if (!b) return null;
  return { name: b.name, fixed: b.fixed, slots: b.slots.map((s) => (s ? mapItem(s) : null)) };
}

function mapBags(b: ServerItemBags): ItemBags {
  return { bags: b.bags.map(mapBag) };
}

function mapActionBinding(a: ServerActionBinding | null): ActionBinding | null {
  if (!a) return null;
  if (a.kind === "item") return { kind: "item", itemId: a.item_id };
  if (a.kind === "skill") return { kind: "skill", skillId: a.skill_id };
  return { kind: "attack" };
}

function mapActionBar(a: ServerActionBar): ActionBar {
  return { slots: a.slots.map(mapActionBinding) };
}

function mapEquipment(e: ServerEquipment): Equipment {
  const m = (i: ServerItem | null): Item | null => (i ? mapItem(i) : null);
  return {
    helmet: m(e.helmet),
    amulet: m(e.amulet),
    chest: m(e.chest),
    belt: m(e.belt),
    gloves: m(e.gloves),
    boots: m(e.boots),
    weapon: m(e.weapon),
    offhand: m(e.offhand),
    ring1: m(e.ring1),
    ring2: m(e.ring2),
  };
}

function mapZoneId(z: string): ZoneId {
  if (z === "wilderness" || z === "burial_grounds") return z;
  return "town";
}

function mapZone(z: ServerZone): Zone {
  return {
    id: mapZoneId(z.id),
    name: z.name,
    kind: z.kind as ZoneKind,
    bounds: { minX: z.bounds.min_x, maxX: z.bounds.max_x, minZ: z.bounds.min_z, maxZ: z.bounds.max_z },
    spawnX: z.spawn_x,
    spawnZ: z.spawn_z,
    waypointX: z.waypoint_x,
    waypointZ: z.waypoint_z,
    enemyTarget: z.enemy_target,
  };
}

function mapEnemy(e: ServerEnemy): Enemy {
  return {
    id: e.id,
    kind: e.kind as EnemyKind,
    zone: mapZoneId(e.zone),
    x: e.x,
    z: e.z,
    hp: e.hp,
    maxHp: e.max_hp,
    damage: e.damage,
    level: e.level,
    moveSpeed: e.move_speed,
    xpReward: e.xp_reward,
    state: e.state as EnemyState,
    targetPlayerId: e.target_player_id,
    attackCooldown: e.attack_cooldown,
    despawnIn: e.despawn_in,
    spawnX: e.spawn_x,
    spawnZ: e.spawn_z,
  };
}

function mapLoot(l: ServerLootDrop): LootDrop {
  return {
    id: l.id,
    item: mapItem(l.item),
    x: l.x,
    z: l.z,
    zone: mapZoneId(l.zone),
    droppedTick: l.dropped_tick,
  };
}

function mapSnapshot(s: ServerPlayerSnapshot) {
  return {
    tick: s.tick,
    elapsedSecs: s.elapsed_secs,
    playerX: s.player.x,
    playerZ: s.player.z,
    gold: s.player.gold,
    inventory: s.player.inventory,
    reputation: s.player.reputation,
    activeMissions: s.player.active_missions.map(mapMission),
    ownedMarketId: s.player.owned_market_id,
    nearestMarketId: s.player.nearest_market_id,
    showTradePanel: s.player.show_trade_panel,
    notification: s.player.notification,
    commodities: s.commodities.map(mapCommodity),
    playerMarkets: s.player_markets.map(mapMarket),
    missionBoard: s.mission_board.map(mapMission),
    tradeHistory: s.player.trade_history.map(mapTradeRecord),
    otherPlayers: s.other_players.map(mapOtherPlayer),
    bags: mapBags(s.player.bags),
    actionBar: mapActionBar(s.player.action_bar),
    equipment: mapEquipment(s.player.equipment),
    level: s.player.level,
    xp: s.player.xp,
    xpToNext: s.player.xp_to_next,
    unspentStatPoints: s.player.unspent_stat_points,
    stats: s.player.stats,
    hp: s.player.hp,
    maxHp: s.player.max_hp,
    mana: s.player.mana,
    maxMana: s.player.max_mana,
    isDead: s.player.is_dead,
    respawnIn: s.player.respawn_in,
    zone: mapZoneId(s.player.zone),
    unlockedWaypoints: s.player.unlocked_waypoints.map(mapZoneId),
    mouseLeft: mapActionBinding(s.player.mouse_left),
    mouseRight: mapActionBinding(s.player.mouse_right),
    classId: s.player.class_id,
    allocatedSkills: s.player.allocated_skills ?? {},
    unspentSkillPoints: s.player.unspent_skill_points ?? 0,
    skillCooldowns: s.player.skill_cooldowns ?? {},
    activeBuffs: s.player.active_buffs ?? {},
    resistances: s.player.resistances ?? { physical: 0, fire: 0, cold: 0, lightning: 0, poison: 0, magical: 0 },
    dots: s.player.dots ?? [],
    zones: s.zones.map(mapZone),
    enemies: s.enemies.map(mapEnemy),
    lootDrops: s.loot_drops.map(mapLoot),
  };
}

function mapDelta(d: ServerDeltaSnapshot): Partial<GameStore> {
  const update: Partial<GameStore> = {
    tick: d.tick,
    elapsedSecs: d.elapsed_secs,
    playerX: d.player.x,
    playerZ: d.player.z,
    gold: d.player.gold,
    inventory: d.player.inventory,
    reputation: d.player.reputation,
    activeMissions: d.player.active_missions.map(mapMission),
    ownedMarketId: d.player.owned_market_id,
    nearestMarketId: d.player.nearest_market_id,
    showTradePanel: d.player.show_trade_panel,
    notification: d.player.notification,
    tradeHistory: d.player.trade_history.map(mapTradeRecord),
    otherPlayers: d.other_players.map(mapOtherPlayer),
    bags: mapBags(d.player.bags),
    actionBar: mapActionBar(d.player.action_bar),
    equipment: mapEquipment(d.player.equipment),
    level: d.player.level,
    xp: d.player.xp,
    xpToNext: d.player.xp_to_next,
    unspentStatPoints: d.player.unspent_stat_points,
    stats: d.player.stats,
    hp: d.player.hp,
    maxHp: d.player.max_hp,
    mana: d.player.mana,
    maxMana: d.player.max_mana,
    isDead: d.player.is_dead,
    respawnIn: d.player.respawn_in,
    zone: mapZoneId(d.player.zone),
    unlockedWaypoints: d.player.unlocked_waypoints.map(mapZoneId),
    mouseLeft: mapActionBinding(d.player.mouse_left),
    mouseRight: mapActionBinding(d.player.mouse_right),
    classId: d.player.class_id,
    allocatedSkills: d.player.allocated_skills ?? {},
    unspentSkillPoints: d.player.unspent_skill_points ?? 0,
    skillCooldowns: d.player.skill_cooldowns ?? {},
    activeBuffs: d.player.active_buffs ?? {},
    resistances: d.player.resistances ?? { physical: 0, fire: 0, cold: 0, lightning: 0, poison: 0, magical: 0 },
    dots: d.player.dots ?? [],
    enemies: d.enemies.map(mapEnemy),
    lootDrops: d.loot_drops.map(mapLoot),
  };

  if (d.player_markets) update.playerMarkets = d.player_markets.map(mapMarket);
  if (d.mission_board) update.missionBoard = d.mission_board.map(mapMission);

  return update;
}

// ── Action Promise Helper ────────────────────────────────────

let actionResolver: ((result: { success: boolean; message: string }) => void) | null = null;
let connectionInitialized = false;

function sendAction(msg: Record<string, unknown>): Promise<{ success: boolean; message: string }> {
  return new Promise<{ success: boolean; message: string }>((resolve) => {
    actionResolver = resolve;
    wsTransport.send(msg);
    setTimeout(() => {
      if (actionResolver === resolve) {
        actionResolver = null;
        resolve({ success: false, message: "Server not responding." });
      }
    }, 5000);
  });
}

// ── Store ────────────────────────────────────────────────────

export const useGameStore = create<GameStore>((set, get) => ({
  tick: 0,
  elapsedSecs: 0,
  playerId: "",
  playerName: "",
  playerX: 0,
  playerZ: 0,
  gold: 500,
  inventory: {},
  reputation: 0,
  activeMissions: [],
  ownedMarketId: null,
  nearestMarketId: null,
  commodities: [],
  playerMarkets: [],
  missionBoard: [],
  tradeHistory: [],
  otherPlayers: [],
  showTradePanel: false,
  notification: "",
  connected: false,
  joining: false,
  bags: { bags: [null, null, null, null, null] },
  actionBar: { slots: [null, null, null, null, null, null, null, null, null] },
  equipment: {
    helmet: null,
    amulet: null,
    chest: null,
    belt: null,
    gloves: null,
    boots: null,
    weapon: null,
    offhand: null,
    ring1: null,
    ring2: null,
  },
  inventoryOpen: false,
  characterOpen: false,
  level: 1,
  xp: 0,
  xpToNext: 100,
  unspentStatPoints: 0,
  stats: { strength: 10, dexterity: 10, vitality: 10, energy: 10 },
  hp: 70,
  maxHp: 70,
  mana: 30,
  maxMana: 30,
  isDead: false,
  respawnIn: 0,
  zone: "town",
  unlockedWaypoints: ["town"],
  zones: [],
  enemies: [],
  lootDrops: [],
  mouseLeft: null,
  mouseRight: null,

  classId: null,
  allocatedSkills: {},
  unspentSkillPoints: 0,
  skillCooldowns: {},
  activeBuffs: {},
  skillTreeOpen: false,
  waypointMenuOpen: false,
  targetEnemyId: null,

  lastError: null,
  hoveredSkillRange: null,

  resistances: { physical: 0, fire: 0, cold: 0, lightning: 0, poison: 0, magical: 0 },
  dots: [],

  initConnection: (playerName: string, serverUrl?: string) => {
    if (connectionInitialized) return;
    connectionInitialized = true;
    set({ joining: true });

    wsTransport.onMessage((data: unknown) => {
      const msg = data as ServerMessage;

      switch (msg.type) {
        case "welcome":
          set({
            playerId: msg.player_id ?? "",
            playerName,
            connected: true,
          });
          break;

        case "state":
          if (msg.snapshot) {
            const mapped = mapSnapshot(msg.snapshot as ServerPlayerSnapshot);
            set(mapped);
          }
          break;

        case "delta":
          if (msg.snapshot) {
            const delta = mapDelta(msg.snapshot as ServerDeltaSnapshot);
            set(delta);
          }
          break;

        case "action_result":
          if (!msg.success && msg.message) {
            set({ lastError: { message: msg.message, ts: Date.now() } });
          }
          if (actionResolver) {
            actionResolver({
              success: msg.success ?? false,
              message: msg.message ?? "",
            });
            actionResolver = null;
          }
          break;

        case "error":
          break;
      }
    });

    wsTransport.connect(serverUrl);

    const joinInterval = setInterval(() => {
      if (wsTransport.isConnected()) {
        clearInterval(joinInterval);
        wsTransport.send({ cmd: "join", name: playerName });
      }
    }, 100);
  },

  sendMove: (dx: number, dz: number) => {
    wsTransport.send({ cmd: "move", dx, dz });
  },

  sendAttack: (enemyId: string, mouseButton: 0 | 1) => {
    return sendAction({ cmd: "attack", enemy_id: enemyId, mouse_button: mouseButton });
  },

  sendPickupLoot: (lootId: string) => {
    return sendAction({ cmd: "pickup_loot", loot_id: lootId });
  },

  sendTravelWaypoint: (zone: ZoneId) => {
    return sendAction({ cmd: "travel_waypoint", zone });
  },

  sendAllocateStat: (stat) => {
    return sendAction({ cmd: "allocate_stat", stat });
  },

  sendSetMouseSkill: (mouseButton: 0 | 1, itemId: string | null) => {
    return sendAction({ cmd: "set_mouse_skill", mouse_button: mouseButton, item_id: itemId });
  },

  sendBindMouseSkill: (mouseButton: 0 | 1, skillId: string | null) => {
    return sendAction({ cmd: "bind_mouse_skill", mouse_button: mouseButton, skill_id: skillId });
  },

  sendChooseClass: (classId: ClassId) => {
    return sendAction({ cmd: "choose_class", class: classId });
  },

  sendAllocateSkill: (skillId: string) => {
    return sendAction({ cmd: "allocate_skill", skill_id: skillId });
  },

  sendCastSkill: (skillId, targetEnemyId, targetX, targetZ) => {
    return sendAction({
      cmd: "cast_skill",
      skill_id: skillId,
      target_enemy_id: targetEnemyId,
      target_x: targetX,
      target_z: targetZ,
    });
  },

  sendCreateMarket: (name: string) => {
    return sendAction({ cmd: "create_market", name });
  },

  sendPostOrder: (commodityId: string, orderType: "buy" | "sell", quantity: number, pricePerUnit: number) => {
    return sendAction({
      cmd: "post_order",
      commodity_id: commodityId,
      order_type: orderType,
      quantity,
      price_per_unit: pricePerUnit,
    });
  },

  sendCancelOrder: (orderId: string) => {
    return sendAction({ cmd: "cancel_order", order_id: orderId });
  },

  sendFillOrder: (marketId: string, orderId: string, quantity: number) => {
    return sendAction({
      cmd: "fill_order",
      market_id: marketId,
      order_id: orderId,
      quantity,
    });
  },

  sendAcceptMission: (missionId: string) => {
    return sendAction({ cmd: "accept_mission", mission_id: missionId });
  },

  sendToggleTradePanel: () => {
    wsTransport.send({ cmd: "toggle_trade_panel" });
  },

  sendCloseTradePanel: () => {
    wsTransport.send({ cmd: "close_trade_panel" });
  },

  sendMoveItem: (srcBag: number, srcSlot: number, dstBag: number, dstSlot: number) => {
    return sendAction({ cmd: "move_item", src_bag: srcBag, src_slot: srcSlot, dst_bag: dstBag, dst_slot: dstSlot });
  },

  sendDropItem: (bag: number, slot: number) => {
    return sendAction({ cmd: "drop_item", bag, slot });
  },

  sendSetActionSlot: (slot: number, itemId: string | null) => {
    return sendAction({ cmd: "set_action_slot", slot, item_id: itemId });
  },

  sendSetActionSlotSkill: (slot: number, skillId: string) => {
    return sendAction({ cmd: "set_action_slot_skill", slot, skill_id: skillId });
  },

  sendUseActionSlot: (slot: number) => {
    return sendAction({ cmd: "use_action_slot", slot });
  },

  sendEquipItem: (bag: number, slot: number, target?: EquipSlotName | null) => {
    return sendAction({ cmd: "equip_item", bag, slot, target: target ?? null });
  },

  sendUnequipItem: (target: EquipSlotName) => {
    return sendAction({ cmd: "unequip_item", target });
  },

  toggleInventory: () => {
    set((s) => ({ inventoryOpen: !s.inventoryOpen }));
  },

  setInventoryOpen: (open: boolean) => {
    set({ inventoryOpen: open });
  },

  toggleCharacter: () => {
    set((s) => ({ characterOpen: !s.characterOpen }));
  },

  setCharacterOpen: (open: boolean) => {
    set({ characterOpen: open });
  },

  toggleSkillTree: () => {
    set((s) => ({ skillTreeOpen: !s.skillTreeOpen }));
  },

  setSkillTreeOpen: (open: boolean) => {
    set({ skillTreeOpen: open });
  },

  toggleWaypointMenu: () => {
    set((s) => ({ waypointMenuOpen: !s.waypointMenuOpen }));
  },

  setWaypointMenuOpen: (open: boolean) => {
    set({ waypointMenuOpen: open });
  },

  setTargetEnemy: (id: string | null) => {
    set({ targetEnemyId: id });
  },

  setHoveredSkillRange: (range: number | null) => {
    set({ hoveredSkillRange: range });
  },

  clearLastError: () => {
    set({ lastError: null });
  },

  cycleTarget: () => {
    const s = get();
    const alive = s.enemies
      .filter((e) => e.state !== "dead" && e.zone === s.zone)
      .map((e) => ({
        id: e.id,
        d: Math.hypot(e.x - s.playerX, e.z - s.playerZ),
      }))
      .sort((a, b) => a.d - b.d);
    if (alive.length === 0) {
      set({ targetEnemyId: null });
      return;
    }
    const idx = alive.findIndex((e) => e.id === s.targetEnemyId);
    const next = alive[(idx + 1) % alive.length];
    set({ targetEnemyId: next.id });
  },
}));