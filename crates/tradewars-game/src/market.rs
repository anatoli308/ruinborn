use serde::{Deserialize, Serialize};
use rand::Rng;
use std::collections::HashMap;

// ── Data Structures ───────────────────────────────────────────

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

/// Per-player state — each connected player has one
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
    pub gold: f64,
    pub inventory: HashMap<String, u32>,
    pub reputation: u32,
    pub nearest_post_id: Option<String>,
    pub show_trade_panel: bool,
    pub trade_history: Vec<TradeOrder>,
    pub notification: String,
}

/// Complete authoritative world state — lives only on the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub tick: u64,
    pub commodities: Vec<Commodity>,
    pub trading_posts: Vec<TradingPost>,
    pub active_events: Vec<MarketEvent>,
    pub players: HashMap<String, PlayerState>,
}

/// Snapshot sent to an individual player (their own data + world)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub tick: u64,
    pub player: PlayerState,
    pub other_players: Vec<OtherPlayer>,
    pub commodities: Vec<Commodity>,
    pub trading_posts: Vec<TradingPost>,
    pub active_events: Vec<MarketEvent>,
}

/// Minimal data about other players (visible to everyone)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherPlayer {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
}

// ── Constants ─────────────────────────────────────────────────

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

pub const POST_INTERACTION_RANGE: f64 = 5.0;
pub const WORLD_BOUND: f64 = 90.0;
const PRICE_HISTORY_MAX: usize = 60;
const STARTING_GOLD: f64 = 10_000.0;

// ── World Initialization ──────────────────────────────────────

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
        commodities,
        trading_posts,
        active_events: Vec::new(),
        players: HashMap::new(),
    }
}

/// Create a new player with starting resources
pub fn create_player(id: &str, name: &str) -> PlayerState {
    PlayerState {
        id: id.to_string(),
        name: name.to_string(),
        x: 0.0,
        z: 0.0,
        gold: STARTING_GOLD,
        inventory: HashMap::new(),
        reputation: 0,
        nearest_post_id: None,
        show_trade_panel: false,
        trade_history: Vec::new(),
        notification: String::new(),
    }
}

/// Build a per-player snapshot of the world for sending over the network
pub fn build_player_snapshot(state: &GameState, player_id: &str) -> Option<PlayerSnapshot> {
    let player = state.players.get(player_id)?.clone();
    let other_players: Vec<OtherPlayer> = state.players.iter()
        .filter(|(id, _)| *id != player_id)
        .map(|(_, p)| OtherPlayer {
            id: p.id.clone(),
            name: p.name.clone(),
            x: p.x,
            z: p.z,
        })
        .collect();

    Some(PlayerSnapshot {
        tick: state.tick,
        player,
        other_players,
        commodities: state.commodities.clone(),
        trading_posts: state.trading_posts.clone(),
        active_events: state.active_events.clone(),
    })
}

// ── Simulation (Server-Authoritative) ─────────────────────────

/// Advance one tick of the world simulation — always runs, never paused
pub fn advance_tick(state: &mut GameState) {
    let mut rng = rand::thread_rng();

    // Decay events
    for ev in &mut state.active_events {
        ev.remaining_ticks -= 1;
    }
    state.active_events.retain(|e| e.remaining_ticks > 0);

    // Clear per-player notifications
    for player in state.players.values_mut() {
        player.notification.clear();
    }

    // Maybe spawn event (3% chance)
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
        let notification = format!("\u{26A1} {}: {}", ev.name, ev.description);
        // Broadcast notification to all players
        for player in state.players.values_mut() {
            player.notification = notification.clone();
        }
        state.active_events.push(ev);
    }

    // Update commodity prices
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

// ── Player Actions ────────────────────────────────────────────

/// Move a specific player by delta, clamped to world bounds
pub fn move_player(state: &mut GameState, player_id: &str, dx: f64, dz: f64) {
    if let Some(player) = state.players.get_mut(player_id) {
        if player.show_trade_panel { return; }

        player.x = (player.x + dx).max(-WORLD_BOUND).min(WORLD_BOUND);
        player.z = (player.z + dz).max(-WORLD_BOUND).min(WORLD_BOUND);

        update_nearest_post(player, &state.trading_posts);
    }
}

fn update_nearest_post(player: &mut PlayerState, trading_posts: &[TradingPost]) {
    let mut nearest: Option<String> = None;
    let mut nearest_dist = f64::INFINITY;

    for post in trading_posts {
        let dist = ((post.x - player.x).powi(2) + (post.z - player.z).powi(2)).sqrt();
        if dist < POST_INTERACTION_RANGE && dist < nearest_dist {
            nearest = Some(post.id.clone());
            nearest_dist = dist;
        }
    }

    player.nearest_post_id = nearest;
}

/// Execute a trade for a specific player — fully validated server-side
pub fn execute_trade(state: &mut GameState, player_id: &str, commodity_id: &str, trade_type: &str, quantity: u32) -> TradeResult {
    let player = match state.players.get_mut(player_id) {
        Some(p) => p,
        None => return TradeResult { success: false, message: "Spieler nicht gefunden!".to_string() },
    };

    if !player.show_trade_panel || player.nearest_post_id.is_none() {
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
            if player.gold < total {
                return TradeResult { success: false, message: "\u{274C} Nicht genug Gold!".to_string() };
            }
            player.gold -= total;
            let entry = player.inventory.entry(commodity_id.to_string()).or_insert(0);
            *entry += quantity;
            player.reputation += quantity;
        }
        "sell" => {
            let owned = player.inventory.get(commodity_id).copied().unwrap_or(0);
            if owned < quantity {
                return TradeResult { success: false, message: "\u{274C} Nicht genug Waren!".to_string() };
            }
            player.gold += total;
            let entry = player.inventory.entry(commodity_id.to_string()).or_insert(0);
            *entry -= quantity;
            player.reputation += quantity;
        }
        _ => return TradeResult { success: false, message: "Unbekannter Handelstyp!".to_string() },
    }

    let name = state.commodities.iter().find(|c| c.id == commodity_id).map(|c| c.name.clone()).unwrap_or_default();
    let msg = if trade_type == "buy" {
        format!("\u{2705} Gekauft: {}x {}", quantity, name)
    } else {
        format!("\u{2705} Verkauft: {}x {}", quantity, name)
    };

    player.notification = msg.clone();
    player.trade_history.push(TradeOrder {
        commodity_id: commodity_id.to_string(),
        trade_type: trade_type.to_string(),
        quantity,
        price_per_unit: price,
        tick: state.tick,
    });

    TradeResult { success: true, message: msg }
}

/// Toggle trade panel for a specific player
pub fn toggle_trade_panel(state: &mut GameState, player_id: &str) {
    if let Some(player) = state.players.get_mut(player_id) {
        if player.nearest_post_id.is_some() || player.show_trade_panel {
            player.show_trade_panel = !player.show_trade_panel;
        }
    }
}

/// Close trade panel for a specific player
pub fn close_trade_panel(state: &mut GameState, player_id: &str) {
    if let Some(player) = state.players.get_mut(player_id) {
        player.show_trade_panel = false;
    }
}
