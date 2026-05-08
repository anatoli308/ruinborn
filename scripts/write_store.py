"""Generate the new gameStore.ts for the community-driven economy model."""
import pathlib

CONTENT = '''import { create } from "zustand";
import { wsTransport } from "../services/wsTransport";
import type {
  Commodity,
  MarketOrder,
  Mission,
  OtherPlayer,
  PlayerMarket,
  ResourceNode,
  TradeRecord,
} from "../types";

// ── Server Snapshot Shape (mirrors Rust PlayerSnapshot) ─────

interface ServerPlayerSnapshot {
  tick: number;
  player: ServerPlayer;
  other_players: ServerOtherPlayer[];
  commodities: ServerCommodity[];
  player_markets: ServerPlayerMarket[];
  resource_nodes: ServerResourceNode[];
  mission_board: ServerMission[];
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
  nearest_node_id: string | null;
  show_trade_panel: boolean;
  trade_history: ServerTradeRecord[];
  notification: string;
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

interface ServerResourceNode {
  id: string;
  commodity_id: string;
  name: string;
  x: number;
  z: number;
  amount: number;
  max_amount: number;
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
  player: ServerPlayer;
  other_players: ServerOtherPlayer[];
  player_markets?: ServerPlayerMarket[];
  resource_nodes?: ServerResourceNode[];
  mission_board?: ServerMission[];
}

// ── Frontend Store (thin client) ─────────────────────────────

interface GameStore {
  // State (read-only mirror of server)
  tick: number;
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
  nearestNodeId: string | null;
  commodities: Commodity[];
  playerMarkets: PlayerMarket[];
  resourceNodes: ResourceNode[];
  missionBoard: Mission[];
  tradeHistory: TradeRecord[];
  otherPlayers: OtherPlayer[];
  showTradePanel: boolean;
  notification: string;
  connected: boolean;

  // Actions (send commands via WebSocket)
  sendMove: (dx: number, dz: number) => void;
  sendGather: () => Promise<{ success: boolean; message: string }>;
  sendCreateMarket: (name: string) => Promise<{ success: boolean; message: string }>;
  sendPostOrder: (commodityId: string, orderType: "buy" | "sell", quantity: number, pricePerUnit: number) => Promise<{ success: boolean; message: string }>;
  sendCancelOrder: (orderId: string) => Promise<{ success: boolean; message: string }>;
  sendFillOrder: (marketId: string, orderId: string, quantity: number) => Promise<{ success: boolean; message: string }>;
  sendAcceptMission: (missionId: string) => Promise<{ success: boolean; message: string }>;
  sendToggleTradePanel: () => void;
  sendCloseTradePanel: () => void;
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

function mapNode(n: ServerResourceNode): ResourceNode {
  return { id: n.id, commodityId: n.commodity_id, name: n.name, x: n.x, z: n.z, amount: n.amount, maxAmount: n.max_amount };
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

function mapSnapshot(s: ServerPlayerSnapshot) {
  return {
    tick: s.tick,
    playerX: s.player.x,
    playerZ: s.player.z,
    gold: s.player.gold,
    inventory: s.player.inventory,
    reputation: s.player.reputation,
    activeMissions: s.player.active_missions.map(mapMission),
    ownedMarketId: s.player.owned_market_id,
    nearestMarketId: s.player.nearest_market_id,
    nearestNodeId: s.player.nearest_node_id,
    showTradePanel: s.player.show_trade_panel,
    notification: s.player.notification,
    commodities: s.commodities.map(mapCommodity),
    playerMarkets: s.player_markets.map(mapMarket),
    resourceNodes: s.resource_nodes.map(mapNode),
    missionBoard: s.mission_board.map(mapMission),
    tradeHistory: s.player.trade_history.map(mapTradeRecord),
    otherPlayers: s.other_players.map(mapOtherPlayer),
  };
}

function mapDelta(d: ServerDeltaSnapshot): Partial<GameStore> {
  const update: Partial<GameStore> = {
    tick: d.tick,
    playerX: d.player.x,
    playerZ: d.player.z,
    gold: d.player.gold,
    inventory: d.player.inventory,
    reputation: d.player.reputation,
    activeMissions: d.player.active_missions.map(mapMission),
    ownedMarketId: d.player.owned_market_id,
    nearestMarketId: d.player.nearest_market_id,
    nearestNodeId: d.player.nearest_node_id,
    showTradePanel: d.player.show_trade_panel,
    notification: d.player.notification,
    tradeHistory: d.player.trade_history.map(mapTradeRecord),
    otherPlayers: d.other_players.map(mapOtherPlayer),
  };

  if (d.player_markets) {
    update.playerMarkets = d.player_markets.map(mapMarket);
  }
  if (d.resource_nodes) {
    update.resourceNodes = d.resource_nodes.map(mapNode);
  }
  if (d.mission_board) {
    update.missionBoard = d.mission_board.map(mapMission);
  }

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
        resolve({ success: false, message: "Server antwortet nicht." });
      }
    }, 5000);
  });
}

// ── Store ────────────────────────────────────────────────────

export const useGameStore = create<GameStore>((set, get) => ({
  tick: 0,
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
  nearestNodeId: null,
  commodities: [],
  playerMarkets: [],
  resourceNodes: [],
  missionBoard: [],
  tradeHistory: [],
  otherPlayers: [],
  showTradePanel: false,
  notification: "",
  connected: false,

  initConnection: (playerName: string, serverUrl?: string) => {
    if (connectionInitialized) return;
    connectionInitialized = true;

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

  sendGather: () => {
    return sendAction({ cmd: "gather" });
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
}));
'''

path = pathlib.Path(r'd:\\projects\\tradewars\\src\\store\\gameStore.ts')
path.write_text(CONTENT.strip(), encoding='utf-8')
print(f"Written {len(CONTENT.strip().splitlines())} lines to {path}")
