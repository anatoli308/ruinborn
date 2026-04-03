import { create } from "zustand";
import { wsTransport } from "../services/wsTransport";
import type { Commodity, MarketEvent, OtherPlayer, TradeOrder, TradingPost } from "../types";

// ── Server Snapshot Shape (mirrors Rust PlayerSnapshot) ─────

interface ServerPlayerSnapshot {
  tick: number;
  player: ServerPlayer;
  other_players: ServerOtherPlayer[];
  commodities: ServerCommodity[];
  trading_posts: ServerTradingPost[];
  active_events: ServerMarketEvent[];
}

interface ServerPlayer {
  id: string;
  name: string;
  x: number;
  z: number;
  gold: number;
  inventory: Record<string, number>;
  reputation: number;
  nearest_post_id: string | null;
  show_trade_panel: boolean;
  trade_history: ServerTradeOrder[];
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
  price: number;
  price_history: number[];
  supply: number;
  demand: number;
  volatility: number;
  category: string;
}

interface ServerTradingPost {
  id: string;
  name: string;
  x: number;
  z: number;
  specialties: string[];
  level: number;
  owned: boolean;
}

interface ServerMarketEvent {
  id: string;
  name: string;
  description: string;
  effects: { commodity_id: string; supply_mod: number; demand_mod: number }[];
  remaining_ticks: number;
}

interface ServerTradeOrder {
  commodity_id: string;
  trade_type: string;
  quantity: number;
  price_per_unit: number;
  tick: number;
}

interface ServerMessage {
  type: string;
  snapshot?: ServerPlayerSnapshot;
  success?: boolean;
  message?: string;
  player_id?: string;
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
  commodities: Commodity[];
  tradingPosts: TradingPost[];
  activeEvents: MarketEvent[];
  tradeHistory: TradeOrder[];
  otherPlayers: OtherPlayer[];
  nearestPostId: string | null;
  showTradePanel: boolean;
  notification: string;
  connected: boolean;

  // Actions (send commands via WebSocket)
  sendMove: (dx: number, dz: number) => void;
  sendTrade: (commodityId: string, type: "buy" | "sell", quantity: number) => Promise<{ success: boolean; message: string }>;
  sendToggleTradePanel: () => void;
  sendCloseTradePanel: () => void;
  initConnection: (playerName: string, serverUrl?: string) => void;
}

function mapSnapshot(s: ServerPlayerSnapshot) {
  return {
    tick: s.tick,
    playerX: s.player.x,
    playerZ: s.player.z,
    gold: s.player.gold,
    inventory: s.player.inventory,
    reputation: s.player.reputation,
    nearestPostId: s.player.nearest_post_id,
    showTradePanel: s.player.show_trade_panel,
    notification: s.player.notification,
    commodities: s.commodities.map((c) => ({
      id: c.id,
      name: c.name,
      icon: c.icon,
      price: c.price,
      priceHistory: c.price_history,
      supply: c.supply,
      demand: c.demand,
      volatility: c.volatility,
      category: c.category,
    })) as Commodity[],
    tradingPosts: s.trading_posts.map((p) => ({
      id: p.id,
      name: p.name,
      x: p.x,
      z: p.z,
      specialties: p.specialties,
      level: p.level,
      owned: p.owned,
    })) as TradingPost[],
    activeEvents: s.active_events.map((e) => ({
      id: e.id,
      name: e.name,
      description: e.description,
      effects: e.effects.map((eff) => ({
        commodityId: eff.commodity_id,
        supplyMod: eff.supply_mod,
        demandMod: eff.demand_mod,
      })),
      remainingTicks: e.remaining_ticks,
    })) as MarketEvent[],
    tradeHistory: s.player.trade_history.map((t) => ({
      commodityId: t.commodity_id,
      type: t.trade_type as "buy" | "sell",
      quantity: t.quantity,
      pricePerUnit: t.price_per_unit,
      tick: t.tick,
    })) as TradeOrder[],
    otherPlayers: s.other_players.map((p) => ({
      id: p.id,
      name: p.name,
      x: p.x,
      z: p.z,
    })) as OtherPlayer[],
  };
}

// Pending trade promise resolver
let tradeResolver: ((result: { success: boolean; message: string }) => void) | null = null;
let connectionInitialized = false;

export const useGameStore = create<GameStore>((set, get) => ({
  tick: 0,
  playerId: "",
  playerName: "",
  playerX: 0,
  playerZ: 0,
  gold: 10_000,
  inventory: {},
  reputation: 0,
  commodities: [],
  tradingPosts: [],
  activeEvents: [],
  tradeHistory: [],
  otherPlayers: [],
  nearestPostId: null,
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
            const mapped = mapSnapshot(msg.snapshot);
            // Preserve tradingPosts reference if unchanged (avoids Trees re-render)
            const current = get();
            if (
              mapped.tradingPosts.length === current.tradingPosts.length &&
              mapped.tradingPosts.every(
                (p, i) =>
                  p.id === current.tradingPosts[i]?.id &&
                  p.x === current.tradingPosts[i]?.x &&
                  p.z === current.tradingPosts[i]?.z
              )
            ) {
              mapped.tradingPosts = current.tradingPosts;
            }
            set(mapped);
          }
          break;

        case "trade_result":
          if (tradeResolver) {
            tradeResolver({
              success: msg.success ?? false,
              message: msg.message ?? "",
            });
            tradeResolver = null;
          }
          break;

        case "error":
          break;
      }
    });

    wsTransport.connect(serverUrl);

    // Wait briefly for connection, then send join
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

  sendTrade: (commodityId: string, type_: "buy" | "sell", quantity: number) => {
    return new Promise<{ success: boolean; message: string }>((resolve) => {
      tradeResolver = resolve;
      wsTransport.send({
        cmd: "trade",
        commodity_id: commodityId,
        trade_type: type_,
        quantity,
      });
      // Timeout after 5 seconds
      setTimeout(() => {
        if (tradeResolver === resolve) {
          tradeResolver = null;
          resolve({ success: false, message: "Server antwortet nicht." });
        }
      }, 5000);
    });
  },

  sendToggleTradePanel: () => {
    wsTransport.send({ cmd: "toggle_trade_panel" });
  },

  sendCloseTradePanel: () => {
    wsTransport.send({ cmd: "close_trade_panel" });
  },
}));
