use serde::{Deserialize, Serialize};
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commodity {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub price: f64,
    pub price_history: Vec<f64>,
    pub supply: f64,
    pub demand: f64,
    pub volatility: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPost {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
    pub specialties: Vec<String>,
    pub level: u32,
    pub owned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: Vec<EventEffect>,
    pub remaining_ticks: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEffect {
    pub commodity_id: String,
    pub supply_mod: f64,
    pub demand_mod: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOrder {
    pub commodity_id: String,
    pub trade_type: String,
    pub quantity: u32,
    pub price_per_unit: f64,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub success: bool,
    pub message: String,
}

/// Complete authoritative game state - lives only in Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub tick: u64,
    pub paused: bool,
    pub speed: f64,
    pub player_x: f64,
    pub player_z: f64,
    pub gold: f64,
    pub inventory: HashMap<String, u32>,
    pub reputation: u32,
    pub commodities: Vec<Commodity>,
    pub trading_posts: Vec<TradingPost>,
    pub active_events: Vec<MarketEvent>,
    pub trade_history: Vec<TradeOrder>,
    pub nearest_post_id: Option<String>,
    pub show_trade_panel: bool,
    pub notification: String,
}

struct EventTemplate {
    name: &'static str,
    desc: &'static str,
    commodity_id: &'static str,
    supply_mod: f64,
    demand_mod: f64,
}

const EVENT_TEMPLATES: &[EventTemplate] = &[
    EventTemplate { name: "Dürre", desc: "Ernteausfälle treiben Lebensmittelpreise hoch!", commodity_id: "wheat", supply_mod: 0.6, demand_mod: 1.2 },
    EventTemplate { name: "Krieg", desc: "Waffennachfrage explodiert!", commodity_id: "weapons", supply_mod: 0.8, demand_mod: 1.5 },
    EventTemplate { name: "Goldfieber", desc: "Edelstein-Fund senkt Preise!", commodity_id: "gems", supply_mod: 1.5, demand_mod: 0.9 },
    EventTemplate { name: "Handelsroute", desc: "Neue Gewürzroute eröffnet!", commodity_id: "spices", supply_mod: 1.4, demand_mod: 1.0 },
    EventTemplate { name: "Sturm", desc: "Holzlieferungen gestört!", commodity_id: "wood", supply_mod: 0.5, demand_mod: 1.1 },
    EventTemplate { name: "Innovation", desc: "Neue Schmiedetechnik!", commodity_id: "iron", supply_mod: 1.0, demand_mod: 1.3 },
    EventTemplate { name: "Luxusboom", desc: "Adel verlangt nach Seide!", commodity_id: "silk", supply_mod: 1.0, demand_mod: 1.4 },
];

const POST_INTERACTION_RANGE: f64 = 5.0;
const WORLD_BOUND: f64 = 90.0;
const PRICE_HISTORY_MAX: usize = 60;

pub fn create_initial_state() -> GameState {
    let commodities = vec![
        Commodity { id: "wheat".into(), name: "Weizen".into(), icon: "\u{1F33E}".into(), price: 10.0, price_history: vec![10.0], supply: 500.0, demand: 450.0, volatility: 0.05, category: "food".into() },
        Commodity { id: "iron".into(), name: "Eisen".into(), icon: "\u{26CF}\u{FE0F}".into(), price: 25.0, price_history: vec![25.0], supply: 200.0, demand: 250.0, volatility: 0.08, category: "material".into() },
        Commodity { id: "silk".into(), name: "Seide".into(), icon: "\u{1F9F5}".into(), price: 80.0, price_history: vec![80.0], supply: 50.0, demand: 70.0, volatility: 0.12, category: "luxury".into() },
        Commodity { id: "weapons".into(), name: "Waffen".into(), icon: "\u{2694}\u{FE0F}".into(), price: 120.0, price_history: vec![120.0], supply: 30.0, demand: 45.0, volatility: 0.15, category: "military".into() },
        Commodity { id: "spices".into(), name: "Gewürze".into(), icon: "\u{1F336}\u{FE0F}".into(), price: 45.0, price_history: vec![45.0], supply: 100.0, demand: 130.0, volatility: 0.10, category: "food".into() },
        Commodity { id: "wood".into(), name: "Holz".into(), icon: "\u{1FAB5}".into(), price: 8.0, price_history: vec![8.0], supply: 800.0, demand: 750.0, volatility: 0.03, category: "material".into() },
        Commodity { id: "gems".into(), name: "Edelsteine".into(), icon: "\u{1F48E}".into(), price: 200.0, price_history: vec![200.0], supply: 15.0, demand: 25.0, volatility: 0.20, category: "luxury".into() },
        Commodity { id: "tools".into(), name: "Werkzeuge".into(), icon: "\u{1F527}".into(), price: 35.0, price_history: vec![35.0], supply: 150.0, demand: 160.0, volatility: 0.06, category: "technology".into() },
    ];

    let trading_posts = vec![
        TradingPost { id: "hafen".into(), name: "Hafen von Elaris".into(), x: 0.0, z: 0.0, specialties: vec!["wheat".into(), "wood".into()], level: 1, owned: true },
        TradingPost { id: "bergwerk".into(), name: "Eisenmine Nordpass".into(), x: 30.0, z: -25.0, specialties: vec!["iron".into(), "tools".into()], level: 1, owned: false },
        TradingPost { id: "oase".into(), name: "Oase Shandara".into(), x: -20.0, z: 35.0, specialties: vec!["silk".into(), "spices".into()], level: 1, owned: false },
        TradingPost { id: "festung".into(), name: "Festung Kragmor".into(), x: 40.0, z: 30.0, specialties: vec!["weapons".into()], level: 2, owned: false },
        TradingPost { id: "markt".into(), name: "Großer Markt Valora".into(), x: -35.0, z: -20.0, specialties: vec!["gems".into(), "silk".into()], level: 1, owned: false },
    ];

    GameState {
        tick: 0,
        paused: false,
        speed: 1.0,
        player_x: 0.0,
        player_z: 0.0,
        gold: 10_000.0,
        inventory: HashMap::new(),
        reputation: 0,
        commodities,
        trading_posts,
        active_events: Vec::new(),
        trade_history: Vec::new(),
        nearest_post_id: None,
        show_trade_panel: false,
        notification: String::new(),
    }
}

/// Advance one tick of the simulation (server-authoritative)
pub fn advance_tick(state: &mut GameState) {
    let mut rng = rand::thread_rng();

    // Decay events
    for ev in &mut state.active_events {
        ev.remaining_ticks -= 1;
    }
    state.active_events.retain(|e| e.remaining_ticks > 0);

    // Maybe spawn event (3% chance)
    state.notification.clear();
    if rng.gen::<f64>() < 0.03 {
        let t = &EVENT_TEMPLATES[rng.gen_range(0..EVENT_TEMPLATES.len())];
        let ev = MarketEvent {
            id: format!("ev_{}", state.tick),
            name: t.name.to_string(),
            description: t.desc.to_string(),
            effects: vec![EventEffect {
                commodity_id: t.commodity_id.to_string(),
                supply_mod: t.supply_mod,
                demand_mod: t.demand_mod,
            }],
            remaining_ticks: 5 + rng.gen_range(0..10),
        };
        state.notification = format!("\u{26A1} {}: {}", ev.name, ev.description);
        state.active_events.push(ev);
    }

    // Update commodities
    for commodity in &mut state.commodities {
        let mut supply_mod = 1.0_f64;
        let mut demand_mod = 1.0_f64;
        for ev in &state.active_events {
            for eff in &ev.effects {
                if eff.commodity_id == commodity.id {
                    supply_mod *= eff.supply_mod;
                    demand_mod *= eff.demand_mod;
                }
            }
        }

        commodity.supply += rng.gen_range(-5.0..5.0);
        commodity.demand += rng.gen_range(-5.0..5.0);
        commodity.supply = commodity.supply.max(1.0);
        commodity.demand = commodity.demand.max(1.0);

        let ratio = (commodity.demand * demand_mod) / (commodity.supply * supply_mod);
        let noise = 1.0 + (rng.gen::<f64>() - 0.5) * 2.0 * commodity.volatility;
        let new_price = (commodity.price * ratio * noise).max(0.5);
        commodity.price = (new_price * 100.0).round() / 100.0;

        commodity.price_history.push(commodity.price);
        if commodity.price_history.len() > PRICE_HISTORY_MAX {
            commodity.price_history.remove(0);
        }
    }

    state.tick += 1;
}

/// Move player by delta, clamped to world bounds
pub fn move_player(state: &mut GameState, dx: f64, dz: f64) {
    if state.show_trade_panel { return; }

    state.player_x = (state.player_x + dx).max(-WORLD_BOUND).min(WORLD_BOUND);
    state.player_z = (state.player_z + dz).max(-WORLD_BOUND).min(WORLD_BOUND);

    update_nearest_post(state);
}

fn update_nearest_post(state: &mut GameState) {
    let mut nearest: Option<String> = None;
    let mut nearest_dist = f64::INFINITY;

    for post in &state.trading_posts {
        let dist = ((post.x - state.player_x).powi(2) + (post.z - state.player_z).powi(2)).sqrt();
        if dist < POST_INTERACTION_RANGE && dist < nearest_dist {
            nearest = Some(post.id.clone());
            nearest_dist = dist;
        }
    }

    state.nearest_post_id = nearest;
}

/// Execute a trade - fully validated server-side
pub fn execute_trade(state: &mut GameState, commodity_id: &str, trade_type: &str, quantity: u32) -> TradeResult {
    if !state.show_trade_panel || state.nearest_post_id.is_none() {
        return TradeResult { success: false, message: "Kein Handelsposten in der Nähe!".to_string() };
    }

    let price = match state.commodities.iter().find(|c| c.id == commodity_id) {
        Some(c) => c.price,
        None => return TradeResult { success: false, message: "Unbekannte Ware!".to_string() },
    };

    if quantity == 0 {
        return TradeResult { success: false, message: "Ungültige Menge!".to_string() };
    }

    let total = price * quantity as f64;

    match trade_type {
        "buy" => {
            if state.gold < total {
                return TradeResult { success: false, message: "\u{274C} Nicht genug Gold!".to_string() };
            }
            state.gold -= total;
            let entry = state.inventory.entry(commodity_id.to_string()).or_insert(0);
            *entry += quantity;
            state.reputation += quantity;
        }
        "sell" => {
            let owned = state.inventory.get(commodity_id).copied().unwrap_or(0);
            if owned < quantity {
                return TradeResult { success: false, message: "\u{274C} Nicht genug Waren!".to_string() };
            }
            state.gold += total;
            let entry = state.inventory.entry(commodity_id.to_string()).or_insert(0);
            *entry -= quantity;
            state.reputation += quantity;
        }
        _ => return TradeResult { success: false, message: "Unbekannter Handelstyp!".to_string() },
    }

    let name = state.commodities.iter().find(|c| c.id == commodity_id).map(|c| c.name.clone()).unwrap_or_default();
    let msg = if trade_type == "buy" {
        format!("\u{2705} Gekauft: {}x {}", quantity, name)
    } else {
        format!("\u{2705} Verkauft: {}x {}", quantity, name)
    };

    state.trade_history.push(TradeOrder {
        commodity_id: commodity_id.to_string(),
        trade_type: trade_type.to_string(),
        quantity,
        price_per_unit: price,
        tick: state.tick,
    });

    TradeResult { success: true, message: msg }
}

pub fn toggle_trade_panel(state: &mut GameState) {
    if state.nearest_post_id.is_some() || state.show_trade_panel {
        state.show_trade_panel = !state.show_trade_panel;
    }
}

pub fn set_paused(state: &mut GameState, paused: bool) {
    state.paused = paused;
}

pub fn set_speed(state: &mut GameState, speed: f64) {
    state.speed = speed.max(0.0).min(10.0);
}
