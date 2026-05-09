"""Generate the new market.rs for the community-driven economy model."""
import pathlib

CONTENT = r'''use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Constants ─────────────────────────────────────────────────

pub const WORLD_BOUND: f64 = 90.0;
pub const INTERACTION_RANGE: f64 = 5.0;
pub const GATHER_RANGE: f64 = 3.0;
pub const STARTING_GOLD: f64 = 500.0;
pub const MARKET_CREATION_COST: f64 = 2000.0;
pub const MAX_ACTIVE_MISSIONS: usize = 3;
pub const MISSION_BOARD_SIZE: usize = 8;

// ── Item Catalog ──────────────────────────────────────────────

/// Commodity definition — what items exist in the world (catalog only, no server-set price)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commodity {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub category: String,
    pub base_value: f64,
}

// ── Player Markets ────────────────────────────────────────────

/// A player-owned market/shop at a world location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMarket {
    pub id: String,
    pub owner_id: String,
    pub owner_name: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
    pub orders: Vec<MarketOrder>,
}

/// A buy or sell order posted on a player market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrder {
    pub id: String,
    pub commodity_id: String,
    pub order_type: String,
    pub quantity: u32,
    pub remaining: u32,
    pub price_per_unit: f64,
    pub created_tick: u64,
}

// ── Resource Nodes ────────────────────────────────────────────

/// A gatherable resource node on the world map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: String,
    pub commodity_id: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
    pub amount: u32,
    pub max_amount: u32,
    pub respawn_ticks: u32,
    pub ticks_until_respawn: u32,
}

// ── Missions ──────────────────────────────────────────────────

/// A mission that can be accepted and completed for rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub title: String,
    pub description: String,
    pub mission_type: String,
    pub commodity_id: Option<String>,
    pub target_quantity: u32,
    pub progress: u32,
    pub reward_gold: f64,
    pub reward_items: HashMap<String, u32>,
    pub reward_reputation: u32,
    pub expires_tick: u64,
}

// ── Trade Records ─────────────────────────────────────────────

/// Record of a completed trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub commodity_id: String,
    pub trade_type: String,
    pub quantity: u32,
    pub price_per_unit: f64,
    pub market_id: String,
    pub tick: u64,
}

// ── Action Result ─────────────────────────────────────────────

/// Result of any player action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
}

// ── Player State ──────────────────────────────────────────────

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
    pub active_missions: Vec<Mission>,
    pub owned_market_id: Option<String>,
    pub nearest_market_id: Option<String>,
    pub nearest_node_id: Option<String>,
    pub show_trade_panel: bool,
    pub trade_history: Vec<TradeRecord>,
    pub notification: String,
}

// ── Game State ────────────────────────────────────────────────

/// Complete authoritative world state — lives only on the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub tick: u64,
    pub commodities: Vec<Commodity>,
    pub player_markets: Vec<PlayerMarket>,
    pub resource_nodes: Vec<ResourceNode>,
    pub mission_board: Vec<Mission>,
    pub players: HashMap<String, PlayerState>,
}

// ── Snapshots ─────────────────────────────────────────────────

/// Full snapshot sent to a player (on join, after major actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub tick: u64,
    pub player: PlayerState,
    pub other_players: Vec<OtherPlayer>,
    pub commodities: Vec<Commodity>,
    pub player_markets: Vec<PlayerMarket>,
    pub resource_nodes: Vec<ResourceNode>,
    pub mission_board: Vec<Mission>,
}

/// Delta snapshot — only contains fields that changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub tick: u64,
    pub player: PlayerState,
    pub other_players: Vec<OtherPlayer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_markets: Option<Vec<PlayerMarket>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_nodes: Option<Vec<ResourceNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mission_board: Option<Vec<Mission>>,
}

/// Minimal data about other players (visible to everyone)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherPlayer {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
}

// ── World Initialization ──────────────────────────────────────

pub fn create_initial_state() -> GameState {
    let commodities = vec![
        Commodity { id: "wheat".into(), name: "Weizen".into(), icon: "\u{1F33E}".into(), category: "food".into(), base_value: 10.0 },
        Commodity { id: "iron".into(), name: "Eisen".into(), icon: "\u{26CF}\u{FE0F}".into(), category: "material".into(), base_value: 25.0 },
        Commodity { id: "silk".into(), name: "Seide".into(), icon: "\u{1F9F5}".into(), category: "luxury".into(), base_value: 80.0 },
        Commodity { id: "weapons".into(), name: "Waffen".into(), icon: "\u{2694}\u{FE0F}".into(), category: "military".into(), base_value: 120.0 },
        Commodity { id: "spices".into(), name: "Gew\u{00FC}rze".into(), icon: "\u{1F336}\u{FE0F}".into(), category: "food".into(), base_value: 45.0 },
        Commodity { id: "wood".into(), name: "Holz".into(), icon: "\u{1FAB5}".into(), category: "material".into(), base_value: 8.0 },
        Commodity { id: "gems".into(), name: "Edelsteine".into(), icon: "\u{1F48E}".into(), category: "luxury".into(), base_value: 200.0 },
        Commodity { id: "tools".into(), name: "Werkzeuge".into(), icon: "\u{1F527}".into(), category: "technology".into(), base_value: 35.0 },
    ];

    let resource_nodes = generate_resource_nodes();

    GameState {
        tick: 0,
        commodities,
        player_markets: Vec::new(),
        resource_nodes,
        mission_board: Vec::new(),
        players: HashMap::new(),
    }
}

fn generate_resource_nodes() -> Vec<ResourceNode> {
    let mut rng = rand::thread_rng();
    let mut nodes = Vec::new();

    let templates: &[(&str, &str, u32, u32, u32, u32)] = &[
        ("wheat", "Weizenfeld",       4, 3, 8, 3),
        ("iron",  "Eisenader",        3, 2, 5, 5),
        ("wood",  "Waldst\u{00FC}ck", 4, 4, 10, 2),
        ("silk",  "Seidenraupe",      2, 1, 3, 8),
        ("spices","Gew\u{00FC}rzstrauch", 3, 2, 5, 4),
        ("gems",  "Edelsteinmine",    2, 1, 2, 10),
        ("tools", "Werkstatt-Ruine",  2, 1, 3, 6),
        ("weapons","Verlassene Schmiede", 2, 1, 2, 8),
    ];

    let mut node_id: u32 = 0;
    for (commodity_id, name_prefix, count, min_amt, max_amt, respawn) in templates {
        for i in 0..*count {
            let x = rng.gen_range(-WORLD_BOUND * 0.85..WORLD_BOUND * 0.85);
            let z = rng.gen_range(-WORLD_BOUND * 0.85..WORLD_BOUND * 0.85);
            let max_amount = rng.gen_range(*min_amt..=*max_amt);
            nodes.push(ResourceNode {
                id: format!("node_{}", node_id),
                commodity_id: commodity_id.to_string(),
                name: format!("{} {}", name_prefix, i + 1),
                x,
                z,
                amount: max_amount,
                max_amount,
                respawn_ticks: *respawn,
                ticks_until_respawn: 0,
            });
            node_id += 1;
        }
    }

    nodes
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
        active_missions: Vec::new(),
        owned_market_id: None,
        nearest_market_id: None,
        nearest_node_id: None,
        show_trade_panel: false,
        trade_history: Vec::new(),
        notification: String::new(),
    }
}

// ── Snapshots ─────────────────────────────────────────────────

/// Build a full snapshot for a player (sent on join + after major actions)
pub fn build_player_snapshot(state: &GameState, player_id: &str) -> Option<PlayerSnapshot> {
    let player = state.players.get(player_id)?.clone();
    let other_players = build_other_players(state, player_id);

    Some(PlayerSnapshot {
        tick: state.tick,
        player,
        other_players,
        commodities: state.commodities.clone(),
        player_markets: state.player_markets.clone(),
        resource_nodes: state.resource_nodes.clone(),
        mission_board: state.mission_board.clone(),
    })
}

/// Build a delta snapshot — player+others always, world data only on economy tick
pub fn build_delta_snapshot(
    state: &GameState,
    player_id: &str,
    last_economy_tick: u64,
) -> Option<DeltaSnapshot> {
    let player = state.players.get(player_id)?.clone();
    let other_players = build_other_players(state, player_id);
    let economy_changed = state.tick > last_economy_tick;

    Some(DeltaSnapshot {
        tick: state.tick,
        player,
        other_players,
        player_markets: if economy_changed { Some(state.player_markets.clone()) } else { None },
        resource_nodes: if economy_changed { Some(state.resource_nodes.clone()) } else { None },
        mission_board: if economy_changed { Some(state.mission_board.clone()) } else { None },
    })
}

fn build_other_players(state: &GameState, player_id: &str) -> Vec<OtherPlayer> {
    state.players.iter()
        .filter(|(id, _)| *id != player_id)
        .map(|(_, p)| OtherPlayer {
            id: p.id.clone(),
            name: p.name.clone(),
            x: p.x,
            z: p.z,
        })
        .collect()
}

// ── Simulation ────────────────────────────────────────────────

/// Advance one economy tick — resource respawn, mission board, cleanup
pub fn advance_tick(state: &mut GameState) {
    let mut rng = rand::thread_rng();

    // Clear per-player notifications
    for player in state.players.values_mut() {
        player.notification.clear();
    }

    // Resource node respawn
    for node in &mut state.resource_nodes {
        if node.amount < node.max_amount {
            if node.ticks_until_respawn == 0 {
                node.amount += 1;
                node.ticks_until_respawn = node.respawn_ticks;
            } else {
                node.ticks_until_respawn -= 1;
            }
        }
    }

    // Remove fully filled orders
    for market in &mut state.player_markets {
        market.orders.retain(|o| o.remaining > 0);
    }

    // Refill mission board
    while state.mission_board.len() < MISSION_BOARD_SIZE {
        if let Some(mission) = generate_random_mission(state.tick, &state.commodities, &mut rng) {
            state.mission_board.push(mission);
        } else {
            break;
        }
    }

    // Expire old missions from board
    state.mission_board.retain(|m| m.expires_tick > state.tick);

    // Check player mission expiry
    for player in state.players.values_mut() {
        let expired: Vec<String> = player.active_missions.iter()
            .filter(|m| m.expires_tick <= state.tick)
            .map(|m| m.id.clone())
            .collect();

        if let Some(first) = expired.first() {
            let title = player.active_missions.iter()
                .find(|m| m.id == *first)
                .map(|m| m.title.clone())
                .unwrap_or_default();
            player.notification = format!("\u{23F0} Mission abgelaufen: {}", title);
        }
        player.active_missions.retain(|m| m.expires_tick > state.tick);
    }

    state.tick += 1;
}

fn generate_random_mission(
    current_tick: u64,
    commodities: &[Commodity],
    rng: &mut impl Rng,
) -> Option<Mission> {
    if commodities.is_empty() {
        return None;
    }

    let commodity = &commodities[rng.gen_range(0..commodities.len())];
    let is_gather = rng.gen_bool(0.6);

    let (title, description, target_quantity, reward_gold, reward_rep) = if is_gather {
        let qty: u32 = rng.gen_range(3..=10);
        let gold = commodity.base_value * qty as f64 * 1.5;
        (
            format!("Sammle {} {}", qty, commodity.name),
            format!("Sammle {} Einheiten {} aus Ressourcenquellen.", qty, commodity.name),
            qty,
            gold,
            qty * 2,
        )
    } else {
        let qty: u32 = rng.gen_range(2..=8);
        let gold = commodity.base_value * qty as f64 * 0.5;
        (
            format!("Verkaufe {} {}", qty, commodity.name),
            format!("Verkaufe {} Einheiten {} an einem Spielermarkt.", qty, commodity.name),
            qty,
            gold,
            qty * 3,
        )
    };

    let mission_type = if is_gather { "gather" } else { "sell" };

    Some(Mission {
        id: format!("mission_{}_{}", current_tick, rng.gen_range(1000..9999u32)),
        title,
        description,
        mission_type: mission_type.to_string(),
        commodity_id: Some(commodity.id.clone()),
        target_quantity,
        progress: 0,
        reward_gold: (reward_gold * 100.0).round() / 100.0,
        reward_items: HashMap::new(),
        reward_reputation: reward_rep,
        expires_tick: current_tick + rng.gen_range(60..180),
    })
}

// ── Player Actions ────────────────────────────────────────────

/// Move a player by delta, clamped to world bounds
pub fn move_player(state: &mut GameState, player_id: &str, dx: f64, dz: f64) {
    let market_positions: Vec<(String, f64, f64)> = state.player_markets.iter()
        .map(|m| (m.id.clone(), m.x, m.z))
        .collect();
    let node_positions: Vec<(String, f64, f64)> = state.resource_nodes.iter()
        .map(|n| (n.id.clone(), n.x, n.z))
        .collect();

    if let Some(player) = state.players.get_mut(player_id) {
        if player.show_trade_panel { return; }

        player.x = (player.x + dx).max(-WORLD_BOUND).min(WORLD_BOUND);
        player.z = (player.z + dz).max(-WORLD_BOUND).min(WORLD_BOUND);

        player.nearest_market_id = find_nearest_in_range(
            player.x, player.z, INTERACTION_RANGE,
            market_positions.iter().map(|(id, x, z)| (id.as_str(), *x, *z)),
        );
        player.nearest_node_id = find_nearest_in_range(
            player.x, player.z, GATHER_RANGE,
            node_positions.iter().map(|(id, x, z)| (id.as_str(), *x, *z)),
        );
    }
}

fn find_nearest_in_range<'a>(
    px: f64, pz: f64, range: f64,
    items: impl Iterator<Item = (&'a str, f64, f64)>,
) -> Option<String> {
    let mut nearest: Option<String> = None;
    let mut nearest_dist = f64::INFINITY;

    for (id, x, z) in items {
        let dist = ((x - px).powi(2) + (z - pz).powi(2)).sqrt();
        if dist < range && dist < nearest_dist {
            nearest = Some(id.to_string());
            nearest_dist = dist;
        }
    }

    nearest
}

/// Gather resources from the nearest node
pub fn gather_resource(state: &mut GameState, player_id: &str) -> ActionResult {
    let node_id = match state.players.get(player_id) {
        Some(p) => match &p.nearest_node_id {
            Some(id) => id.clone(),
            None => return ActionResult { success: false, message: "Keine Ressource in der N\u{00E4}he!".into() },
        },
        None => return ActionResult { success: false, message: "Spieler nicht gefunden!".into() },
    };

    let node = match state.resource_nodes.iter_mut().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return ActionResult { success: false, message: "Ressource nicht gefunden!".into() },
    };

    if node.amount == 0 {
        return ActionResult { success: false, message: "Ressource ersch\u{00F6}pft! Warte auf Respawn.".into() };
    }

    node.amount -= 1;
    let commodity_id = node.commodity_id.clone();

    let player = state.players.get_mut(player_id).unwrap();
    let entry = player.inventory.entry(commodity_id.clone()).or_insert(0);
    *entry += 1;

    // Check gather missions
    let mut completed_missions: Vec<String> = Vec::new();
    for mission in &mut player.active_missions {
        if mission.mission_type == "gather" && mission.commodity_id.as_deref() == Some(&commodity_id) {
            mission.progress += 1;
            if mission.progress >= mission.target_quantity {
                completed_missions.push(mission.id.clone());
            }
        }
    }
    for mid in &completed_missions {
        complete_mission_for_player(player, mid);
    }

    let commodity_name = state.commodities.iter()
        .find(|c| c.id == commodity_id)
        .map(|c| c.name.clone())
        .unwrap_or_default();

    ActionResult {
        success: true,
        message: format!("\u{2705} 1x {} gesammelt", commodity_name),
    }
}

/// Create a player-owned market at the player's current position
pub fn create_market(state: &mut GameState, player_id: &str, name: &str) -> ActionResult {
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Spieler nicht gefunden!".into() },
    };

    if player.owned_market_id.is_some() {
        return ActionResult { success: false, message: "Du besitzt bereits einen Markt!".into() };
    }

    if player.gold < MARKET_CREATION_COST {
        return ActionResult {
            success: false,
            message: format!("\u{274C} Nicht genug Gold! Ben\u{00F6}tigt: {}", MARKET_CREATION_COST),
        };
    }

    let market_id = format!("market_{}_{}", player_id.chars().take(8).collect::<String>(), state.tick);
    let market = PlayerMarket {
        id: market_id.clone(),
        owner_id: player_id.to_string(),
        owner_name: player.name.clone(),
        name: name.to_string(),
        x: player.x,
        z: player.z,
        orders: Vec::new(),
    };

    state.player_markets.push(market);

    let player = state.players.get_mut(player_id).unwrap();
    player.gold -= MARKET_CREATION_COST;
    player.owned_market_id = Some(market_id);
    player.notification = format!("\u{1F3EA} Markt '{}' er\u{00F6}ffnet!", name);

    ActionResult { success: true, message: format!("\u{1F3EA} Markt '{}' er\u{00F6}ffnet!", name) }
}

/// Post a buy or sell order on the player's own market
pub fn post_order(
    state: &mut GameState,
    player_id: &str,
    commodity_id: &str,
    order_type: &str,
    quantity: u32,
    price_per_unit: f64,
) -> ActionResult {
    if quantity == 0 {
        return ActionResult { success: false, message: "Ung\u{00FC}ltige Menge!".into() };
    }
    if price_per_unit <= 0.0 {
        return ActionResult { success: false, message: "Ung\u{00FC}ltiger Preis!".into() };
    }
    if !state.commodities.iter().any(|c| c.id == commodity_id) {
        return ActionResult { success: false, message: "Unbekannte Ware!".into() };
    }

    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Spieler nicht gefunden!".into() },
    };

    let market_id = match &player.owned_market_id {
        Some(id) => id.clone(),
        None => return ActionResult { success: false, message: "Du besitzt keinen Markt!".into() },
    };

    // For sell orders: check inventory
    if order_type == "sell" {
        let owned = player.inventory.get(commodity_id).copied().unwrap_or(0);
        if owned < quantity {
            return ActionResult { success: false, message: "\u{274C} Nicht genug Waren im Inventar!".into() };
        }
    }

    // For buy orders: check gold
    if order_type == "buy" {
        let total_cost = price_per_unit * quantity as f64;
        if player.gold < total_cost {
            return ActionResult { success: false, message: "\u{274C} Nicht genug Gold f\u{00FC}r Kauforder!".into() };
        }
    }

    // Reserve resources
    let player = state.players.get_mut(player_id).unwrap();
    if order_type == "sell" {
        let entry = player.inventory.entry(commodity_id.to_string()).or_insert(0);
        *entry -= quantity;
    } else if order_type == "buy" {
        player.gold -= price_per_unit * quantity as f64;
    }

    let order_id = format!("order_{}_{}", state.tick, rand::thread_rng().gen_range(1000..9999u32));

    let market = match state.player_markets.iter_mut().find(|m| m.id == market_id) {
        Some(m) => m,
        None => return ActionResult { success: false, message: "Markt nicht gefunden!".into() },
    };

    market.orders.push(MarketOrder {
        id: order_id,
        commodity_id: commodity_id.to_string(),
        order_type: order_type.to_string(),
        quantity,
        remaining: quantity,
        price_per_unit: (price_per_unit * 100.0).round() / 100.0,
        created_tick: state.tick,
    });

    let commodity_name = state.commodities.iter()
        .find(|c| c.id == commodity_id)
        .map(|c| c.name.clone())
        .unwrap_or_default();
    let type_label = if order_type == "buy" { "Kauforder" } else { "Verkaufsorder" };

    ActionResult {
        success: true,
        message: format!("\u{1F4CB} {} erstellt: {}x {} @ {:.2}g", type_label, quantity, commodity_name, price_per_unit),
    }
}

/// Cancel an order on the player's own market (refund reserved resources)
pub fn cancel_order(state: &mut GameState, player_id: &str, order_id: &str) -> ActionResult {
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Spieler nicht gefunden!".into() },
    };

    let market_id = match &player.owned_market_id {
        Some(id) => id.clone(),
        None => return ActionResult { success: false, message: "Du besitzt keinen Markt!".into() },
    };

    let market = match state.player_markets.iter_mut().find(|m| m.id == market_id) {
        Some(m) => m,
        None => return ActionResult { success: false, message: "Markt nicht gefunden!".into() },
    };

    let order = match market.orders.iter().find(|o| o.id == order_id) {
        Some(o) => o.clone(),
        None => return ActionResult { success: false, message: "Order nicht gefunden!".into() },
    };

    // Refund reserved resources
    let player = state.players.get_mut(player_id).unwrap();
    if order.order_type == "sell" {
        let entry = player.inventory.entry(order.commodity_id.clone()).or_insert(0);
        *entry += order.remaining;
    } else if order.order_type == "buy" {
        player.gold += order.price_per_unit * order.remaining as f64;
    }

    let market = state.player_markets.iter_mut().find(|m| m.id == market_id).unwrap();
    market.orders.retain(|o| o.id != order_id);

    ActionResult { success: true, message: "\u{1F5D1}\u{FE0F} Order storniert.".into() }
}

/// Fill an order at another player's market
pub fn fill_order(
    state: &mut GameState,
    player_id: &str,
    market_id: &str,
    order_id: &str,
    quantity: u32,
) -> ActionResult {
    if quantity == 0 {
        return ActionResult { success: false, message: "Ung\u{00FC}ltige Menge!".into() };
    }

    // Check player is near the market
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Spieler nicht gefunden!".into() },
    };

    if player.nearest_market_id.as_deref() != Some(market_id) {
        return ActionResult { success: false, message: "Nicht in Reichweite des Marktes!".into() };
    }

    // Can't fill own orders
    if player.owned_market_id.as_deref() == Some(market_id) {
        return ActionResult { success: false, message: "Eigene Orders k\u{00F6}nnen nicht gef\u{00FC}llt werden!".into() };
    }

    // Find order and extract info
    let market = match state.player_markets.iter().find(|m| m.id == market_id) {
        Some(m) => m,
        None => return ActionResult { success: false, message: "Markt nicht gefunden!".into() },
    };

    let order = match market.orders.iter().find(|o| o.id == order_id) {
        Some(o) => o,
        None => return ActionResult { success: false, message: "Order nicht gefunden!".into() },
    };

    let fill_qty = quantity.min(order.remaining);
    let total_price = order.price_per_unit * fill_qty as f64;
    let commodity_id = order.commodity_id.clone();
    let order_type = order.order_type.clone();
    let market_owner_id = market.owner_id.clone();
    let order_price = order.price_per_unit;

    // Validate the filling player
    let player = state.players.get(player_id).unwrap();
    if order_type == "sell" {
        if player.gold < total_price {
            return ActionResult { success: false, message: "\u{274C} Nicht genug Gold!".into() };
        }
    } else {
        let owned = player.inventory.get(&commodity_id).copied().unwrap_or(0);
        if owned < fill_qty {
            return ActionResult { success: false, message: "\u{274C} Nicht genug Waren!".into() };
        }
    }

    // Update the order
    let market = state.player_markets.iter_mut().find(|m| m.id == market_id).unwrap();
    let order = market.orders.iter_mut().find(|o| o.id == order_id).unwrap();
    order.remaining -= fill_qty;

    // Update the filling player
    let player = state.players.get_mut(player_id).unwrap();
    if order_type == "sell" {
        player.gold -= total_price;
        let entry = player.inventory.entry(commodity_id.clone()).or_insert(0);
        *entry += fill_qty;
    } else {
        player.gold += total_price;
        let entry = player.inventory.entry(commodity_id.clone()).or_insert(0);
        *entry -= fill_qty;
    }

    let player_trade_type = if order_type == "sell" { "buy" } else { "sell" };
    player.trade_history.push(TradeRecord {
        commodity_id: commodity_id.clone(),
        trade_type: player_trade_type.to_string(),
        quantity: fill_qty,
        price_per_unit: order_price,
        market_id: market_id.to_string(),
        tick: state.tick,
    });
    player.reputation += fill_qty;

    // Check sell missions for filling player
    if player_trade_type == "sell" {
        check_sell_missions(player, &commodity_id, fill_qty);
    }

    // Update the market owner
    if let Some(owner) = state.players.get_mut(&market_owner_id) {
        if order_type == "sell" {
            owner.gold += total_price;
        } else {
            let entry = owner.inventory.entry(commodity_id.clone()).or_insert(0);
            *entry += fill_qty;
        }
        owner.reputation += fill_qty;

        let owner_trade_type = if order_type == "sell" { "sell" } else { "buy" };
        owner.trade_history.push(TradeRecord {
            commodity_id: commodity_id.clone(),
            trade_type: owner_trade_type.to_string(),
            quantity: fill_qty,
            price_per_unit: order_price,
            market_id: market_id.to_string(),
            tick: state.tick,
        });

        if owner_trade_type == "sell" {
            check_sell_missions(owner, &commodity_id, fill_qty);
        }
    }

    let commodity_name = state.commodities.iter()
        .find(|c| c.id == commodity_id)
        .map(|c| c.name.clone())
        .unwrap_or_default();
    let action = if order_type == "sell" { "Gekauft" } else { "Verkauft" };

    ActionResult {
        success: true,
        message: format!("\u{2705} {}: {}x {} @ {:.2}g", action, fill_qty, commodity_name, order_price),
    }
}

/// Accept a mission from the board
pub fn accept_mission(state: &mut GameState, player_id: &str, mission_id: &str) -> ActionResult {
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Spieler nicht gefunden!".into() },
    };

    if player.active_missions.len() >= MAX_ACTIVE_MISSIONS {
        return ActionResult {
            success: false,
            message: format!("\u{274C} Max. {} aktive Missionen!", MAX_ACTIVE_MISSIONS),
        };
    }

    let mission_idx = match state.mission_board.iter().position(|m| m.id == mission_id) {
        Some(i) => i,
        None => return ActionResult { success: false, message: "Mission nicht gefunden!".into() },
    };

    let mission = state.mission_board.remove(mission_idx);
    let title = mission.title.clone();

    let player = state.players.get_mut(player_id).unwrap();
    player.active_missions.push(mission);
    player.notification = format!("\u{1F4DC} Mission angenommen: {}", title);

    ActionResult { success: true, message: format!("\u{1F4DC} Mission angenommen: {}", title) }
}

/// Toggle trade panel for a specific player
pub fn toggle_trade_panel(state: &mut GameState, player_id: &str) {
    if let Some(player) = state.players.get_mut(player_id) {
        if player.nearest_market_id.is_some() || player.show_trade_panel {
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

// ── Helpers ───────────────────────────────────────────────────

fn check_sell_missions(player: &mut PlayerState, commodity_id: &str, quantity: u32) {
    let mut completed: Vec<String> = Vec::new();
    for mission in &mut player.active_missions {
        if mission.mission_type == "sell" && mission.commodity_id.as_deref() == Some(commodity_id) {
            mission.progress += quantity;
            if mission.progress >= mission.target_quantity {
                completed.push(mission.id.clone());
            }
        }
    }
    for mid in &completed {
        complete_mission_for_player(player, mid);
    }
}

fn complete_mission_for_player(player: &mut PlayerState, mission_id: &str) {
    let mission = match player.active_missions.iter().find(|m| m.id == mission_id) {
        Some(m) => m.clone(),
        None => return,
    };

    player.gold += mission.reward_gold;
    player.reputation += mission.reward_reputation;
    for (item_id, qty) in &mission.reward_items {
        let entry = player.inventory.entry(item_id.clone()).or_insert(0);
        *entry += qty;
    }

    player.notification = format!(
        "\u{1F389} Mission abgeschlossen: {} (+{:.0}g, +{} Rep)",
        mission.title, mission.reward_gold, mission.reward_reputation
    );

    player.active_missions.retain(|m| m.id != mission_id);
}
'''

path = pathlib.Path(r'd:\projects\tradewars\crates\ruinborn-game\src\market.rs')
path.write_text(CONTENT.strip(), encoding='utf-8')
print(f"Written {len(CONTENT.strip().splitlines())} lines to {path}")
