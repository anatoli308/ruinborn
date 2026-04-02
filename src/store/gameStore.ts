import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Commodity, MarketEvent, TradeOrder, TradingPost } from "../types";

// ── Server State Shape (mirrors Rust GameState) ─────────────

interface ServerGameState {
  tick: number;
  paused: boolean;
  speed: number;
  player_x: number;
  player_z: number;
  gold: number;
  inventory: Record<string, number>;
  reputation: number;
  commodities: ServerCommodity[];
  trading_posts: ServerTradingPost[];
  active_events: ServerMarketEvent[];
  trade_history: ServerTradeOrder[];
  nearest_post_id: string | null;
  show_trade_panel: boolean;
  notification: string;
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

// ── Frontend Store (thin client) ─────────────────────────────

interface GameStore {
  // State (read-only mirror of Rust)
  tick: number;
  paused: boolean;
  speed: number;
  playerX: number;
  playerZ: number;
  gold: number;
  inventory: Record<string, number>;
  reputation: number;
  commodities: Commodity[];
  tradingPosts: TradingPost[];
  activeEvents: MarketEvent[];
  tradeHistory: TradeOrder[];
  nearestPostId: string | null;
  showTradePanel: boolean;
  notification: string;
  connected: boolean;

  // Actions (send commands to Rust)
  sendMove: (dx: number, dz: number) => void;
  sendTrade: (commodityId: string, type: "buy" | "sell", quantity: number) => Promise<{ success: boolean; message: string }>;
  sendToggleTradePanel: () => void;
  sendCloseTradePanel: () => void;
  sendSetPaused: (paused: boolean) => void;
  sendSetSpeed: (speed: number) => void;
  applyServerState: (s: ServerGameState) => void;
  initListener: () => void;
}

function mapServerState(s: ServerGameState) {
  return {
    tick: s.tick,
    paused: s.paused,
    speed: s.speed,
    playerX: s.player_x,
    playerZ: s.player_z,
    gold: s.gold,
    inventory: s.inventory,
    reputation: s.reputation,
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
    tradeHistory: s.trade_history.map((t) => ({
      commodityId: t.commodity_id,
      type: t.trade_type as "buy" | "sell",
      quantity: t.quantity,
      pricePerUnit: t.price_per_unit,
      tick: t.tick,
    })) as TradeOrder[],
    nearestPostId: s.nearest_post_id,
    showTradePanel: s.show_trade_panel,
    notification: s.notification,
  };
}

export const useGameStore = create<GameStore>((set, get) => ({
  tick: 0,
  paused: false,
  speed: 1,
  playerX: 0,
  playerZ: 0,
  gold: 10_000,
  inventory: {},
  reputation: 0,
  commodities: [],
  tradingPosts: [],
  activeEvents: [],
  tradeHistory: [],
  nearestPostId: null,
  showTradePanel: false,
  notification: "",
  connected: false,

  applyServerState: (s: ServerGameState) => {
    set(mapServerState(s));
  },

  initListener: () => {
    if (get().connected) return;
    set({ connected: true });

    // Listen for realtime state pushes from Rust tick loop
    listen<ServerGameState>("game-state", (event) => {
      set(mapServerState(event.payload));
    });

    // Fetch initial state
    invoke<ServerGameState>("get_game_state").then((s) => {
      set(mapServerState(s));
    });
  },

  sendMove: (dx: number, dz: number) => {
    invoke("move_player", { dx, dz }).catch(console.error);
  },

  sendTrade: async (commodityId: string, type_: "buy" | "sell", quantity: number) => {
    try {
      const result = await invoke<{ success: boolean; message: string }>("execute_trade", {
        commodityId,
        tradeType: type_,
        quantity,
      });
      return result;
    } catch (e) {
      return { success: false, message: String(e) };
    }
  },

  sendToggleTradePanel: () => {
    invoke("toggle_trade_panel").catch(console.error);
  },

  sendCloseTradePanel: () => {
    invoke("close_trade_panel").catch(console.error);
  },

  sendSetPaused: (paused: boolean) => {
    invoke("set_paused", { paused }).catch(console.error);
  },

  sendSetSpeed: (speed: number) => {
    invoke("set_speed", { speed }).catch(console.error);
  },
}));
