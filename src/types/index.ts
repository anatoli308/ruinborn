export type CommodityCategory = "food" | "material" | "luxury" | "military" | "technology";

export interface Commodity {
  id: string;
  name: string;
  icon: string;
  price: number;
  priceHistory: number[];
  supply: number;
  demand: number;
  volatility: number;
  category: CommodityCategory;
}

export interface TradingPost {
  id: string;
  name: string;
  x: number;
  z: number;
  specialties: string[];
  level: number;
  owned: boolean;
}

export interface MarketEvent {
  id: string;
  name: string;
  description: string;
  effects: { commodityId: string; supplyMod: number; demandMod: number }[];
  remainingTicks: number;
}

export interface TradeOrder {
  commodityId: string;
  type: "buy" | "sell";
  quantity: number;
  pricePerUnit: number;
  tick: number;
}
