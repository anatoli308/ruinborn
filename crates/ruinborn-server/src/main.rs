use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use sea_orm::DatabaseConnection;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;

use ruinborn_game::{self as game, EquipSlotName, GameState};
use ruinborn_protocol::{ClientMessage, ServerMessage};

mod db_sea;
mod entity;
use db_sea as db;

/// Map a JSON string ("helmet", "ring1", ...) to the typed `EquipSlotName`.
fn parse_equip_slot(name: &str) -> Option<EquipSlotName> {
    match name {
        "helmet" => Some(EquipSlotName::Helmet),
        "amulet" => Some(EquipSlotName::Amulet),
        "chest" => Some(EquipSlotName::Chest),
        "belt" => Some(EquipSlotName::Belt),
        "gloves" => Some(EquipSlotName::Gloves),
        "boots" => Some(EquipSlotName::Boots),
        "weapon" => Some(EquipSlotName::Weapon),
        "offhand" => Some(EquipSlotName::Offhand),
        "ring1" => Some(EquipSlotName::Ring1),
        "ring2" => Some(EquipSlotName::Ring2),
        _ => None,
    }
}

/// Server configuration
const BIND_ADDR: &str = "0.0.0.0:9000";
/// Single world tick — drives combat simulation and state broadcast (20 Hz).
/// This is our GameClock equivalent of the C++ reference server.
const WORLD_TICK_MS: u64 = 50;
/// Persist game state to DB every N world ticks (= 30 s @ 20 Hz).
const DB_SAVE_TICKS: u64 = 600;

/// Per-connection sender handle
type Tx = mpsc::UnboundedSender<Message>;

/// Shared server state
struct Server {
    game: RwLock<GameState>,
    clients: RwLock<HashMap<String, ClientHandle>>,
    db: DatabaseConnection,
}

struct ClientHandle {
    tx: Tx,
    player_id: String,
    /// Last economy tick this client received commodity/event data for
    last_economy_tick: u64,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load .env file (optional — no error if missing)
    let _ = dotenvy::dotenv();

    // Connect to PostgreSQL
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://user:pass@localhost/ruinborn)");

    let db = db::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    db::ensure_schema(&db)
        .await
        .expect("Failed to ensure database schema");

    // Load or create initial game state
    let initial_state = {
        let fresh = game::create_initial_state();
        match db::load_game_state(&db, fresh.commodities.clone()).await {
            Ok(Some(loaded)) => loaded,
            Ok(None) => {
                // First run — save the initial state
                if let Err(e) = db::save_game_state(&db, &fresh).await {
                    error!("Failed to save initial state: {}", e);
                }
                fresh
            }
            Err(e) => {
                error!("Failed to load game state: {} — starting fresh", e);
                fresh
            }
        }
    };

    let server = Arc::new(Server {
        game: RwLock::new(initial_state),
        clients: RwLock::new(HashMap::new()),
        db,
    });

    // Start the tick loop
    let tick_server = Arc::clone(&server);
    tokio::spawn(async move {
        tick_loop(tick_server).await;
    });

    // Bind TCP listener
    let listener = TcpListener::bind(BIND_ADDR).await.expect("Failed to bind");
    info!("Ruinborn server listening on {}", BIND_ADDR);

    while let Ok((stream, addr)) = listener.accept().await {
        let server = Arc::clone(&server);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(server, stream, addr).await {
                error!("Connection error from {}: {}", addr, e);
            }
        });
    }
}

// ── World Tick ────────────────────────────────────────────────

/// Single 20 Hz world tick — advances combat simulation (enemy AI, cooldowns,
/// DoTs, respawn timers) and broadcasts state to all clients. The 1 Hz
/// economy tick has been removed (YAGNI); reactivate `advance_economy_tick`
/// in `ruinborn-game` if/when missions, market decay, or population
/// maintenance come back online.
async fn tick_loop(server: Arc<Server>) {
    let mut interval = tokio::time::interval(Duration::from_millis(WORLD_TICK_MS));
    let mut tick_count: u64 = 0;

    loop {
        interval.tick().await;
        tick_count = tick_count.wrapping_add(1);

        // Combat sim every world tick: AI, cooldowns, DoTs, respawn timers.
        {
            let mut game = server.game.write().await;
            game::advance_combat_tick(&mut game);
        }

        // Periodic DB save — read lock only, does not block the next sim step.
        if tick_count % DB_SAVE_TICKS == 0 {
            let game = server.game.read().await;
            if let Err(e) = db::save_game_state(&server.db, &game).await {
                error!("Failed to save game state: {}", e);
            }
        }

        broadcast_state(&server).await;
    }
}

/// Send each connected player a delta snapshot (only changed data)
async fn broadcast_state(server: &Server) {
    let game = server.game.read().await;
    let mut clients = server.clients.write().await;

    for (_, client) in clients.iter_mut() {
        if let Some(snapshot) = game::build_delta_snapshot(&game, &client.player_id, client.last_economy_tick) {
            // Update last_economy_tick if world data was included
            if snapshot.player_markets.is_some() {
                client.last_economy_tick = game.tick;
            }
            let msg = ServerMessage::Delta { snapshot };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = client.tx.send(Message::Text(json.into()));
            }
        }
    }
}

// ── Connection Handler ────────────────────────────────────────

async fn handle_connection(
    server: Arc<Server>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    info!("New WebSocket connection from {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let conn_id = uuid::Uuid::new_v4().to_string();
    let mut player_id: Option<String> = None;

    // Outbound message forwarder
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Inbound message handler
    while let Some(msg) = ws_receiver.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                warn!("WebSocket read error from {}: {}", addr, e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                let client_msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("Invalid message from {}: {}", addr, e);
                        continue;
                    }
                };

                match client_msg {
                    ClientMessage::Join { name } => {
                        let pid = uuid::Uuid::new_v4().to_string();
                        info!("Player '{}' joined as {} from {}", name, pid, addr);

                        // Create player in game state
                        {
                            let mut game = server.game.write().await;
                            let new_player = game::create_player(&pid, &name, &game.zones.clone());
                            game.players.insert(pid.clone(), new_player);
                        }

                        // Register client handle
                        {
                            let mut clients = server.clients.write().await;
                            clients.insert(conn_id.clone(), ClientHandle {
                                tx: tx.clone(),
                                player_id: pid.clone(),
                                last_economy_tick: 0,
                            });
                        }

                        player_id = Some(pid.clone());

                        // Send welcome + initial state
                        let welcome = ServerMessage::Welcome { player_id: pid.clone() };
                        if let Ok(json) = serde_json::to_string(&welcome) {
                            let _ = tx.send(Message::Text(json.into()));
                        }

                        let game = server.game.read().await;
                        if let Some(snapshot) = game::build_player_snapshot(&game, &pid) {
                            let state_msg = ServerMessage::State { snapshot };
                            if let Ok(json) = serde_json::to_string(&state_msg) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                        }
                    }

                    ClientMessage::Move { dx, dz } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            game::move_player(&mut game, pid, dx, dz);
                        }
                    }

                    ClientMessage::Gather => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::gather_resource(&mut game, pid);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::CreateMarket { name } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::create_market(&mut game, pid, &name);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::PostOrder { commodity_id, order_type, quantity, price_per_unit } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::post_order(&mut game, pid, &commodity_id, &order_type, quantity, price_per_unit);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::CancelOrder { order_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::cancel_order(&mut game, pid, &order_id);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::FillOrder { market_id, order_id, quantity } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::fill_order(&mut game, pid, &market_id, &order_id, quantity);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::AcceptMission { mission_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::accept_mission(&mut game, pid, &mission_id);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::ToggleTradePanel => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            game::toggle_trade_panel(&mut game, pid);
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::CloseTradePanel => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            game::close_trade_panel(&mut game, pid);
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::MoveItem { src_bag, src_slot, dst_bag, dst_slot } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::move_item(&mut game, pid, src_bag, src_slot, dst_bag, dst_slot);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::DropItem { bag, slot } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::drop_item(&mut game, pid, bag, slot);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::SetActionSlot { slot, item_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::set_action_slot(&mut game, pid, slot, item_id);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::SetActionSlotSkill { slot, skill_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::set_action_slot_skill(&mut game, pid, slot, skill_id);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::UseActionSlot { slot } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::use_action_slot(&mut game, pid, slot);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::EquipItem { bag, slot, target } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let target_slot = target
                                .as_deref()
                                .and_then(parse_equip_slot);
                            let result = game::equip_item(&mut game, pid, bag, slot, target_slot);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::UnequipItem { target } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let target_slot = match parse_equip_slot(&target) {
                                Some(t) => t,
                                None => {
                                    let resp = ServerMessage::ActionResult {
                                        success: false,
                                        message: "\u{274C} Unknown equipment slot.".into(),
                                    };
                                    if let Ok(json) = serde_json::to_string(&resp) {
                                        let _ = tx.send(Message::Text(json.into()));
                                    }
                                    continue;
                                }
                            };
                            let result = game::unequip_item(&mut game, pid, target_slot);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::Attack { enemy_id, mouse_button } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            // Resolve mouse binding: if a Skill is bound, cast it; otherwise basic attack.
                            let bound_skill: Option<String> = game
                                .players
                                .get(pid)
                                .and_then(|p| {
                                    let b = if mouse_button == 1 { &p.mouse_right } else { &p.mouse_left };
                                    match b {
                                        Some(game::ActionBinding::Skill { skill_id }) => Some(skill_id.clone()),
                                        _ => None,
                                    }
                                });
                            let result = if let Some(skill_id) = bound_skill {
                                game::cast_skill(&mut game, pid, &skill_id, Some(&enemy_id), None, None)
                            } else {
                                game::player_attack(&mut game, pid, &enemy_id)
                            };
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                        }
                    }

                    ClientMessage::PickupLoot { loot_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::pickup_loot(&mut game, pid, &loot_id);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if result.success {
                                if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                    let msg = ServerMessage::State { snapshot };
                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        let _ = tx.send(Message::Text(json.into()));
                                    }
                                }
                            }
                        }
                    }

                    ClientMessage::TravelWaypoint { zone } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            // Phase 6: zones are data-driven — just pass the
                            // string through. `travel_waypoint` rejects
                            // unknown ids with a friendly error message.
                            let target = game::ZoneId::from(zone.as_str());
                            let result = game::travel_waypoint(&mut game, pid, &target);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                        }
                    }

                    ClientMessage::AllocateStat { stat } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::allocate_stat(&mut game, pid, &stat);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                        }
                    }

                    ClientMessage::SetMouseSkill { mouse_button, item_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let binding = match item_id {
                                Some(id) => Some(game::ActionBinding::Item { item_id: id }),
                                None => Some(game::ActionBinding::Attack),
                            };
                            let result = game::set_mouse_skill(&mut game, pid, mouse_button, binding);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                        }
                    }

                    ClientMessage::BindMouseSkill { mouse_button, skill_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let binding = match skill_id {
                                Some(id) => Some(game::ActionBinding::Skill { skill_id: id }),
                                None => Some(game::ActionBinding::Attack),
                            };
                            let result = game::set_mouse_skill(&mut game, pid, mouse_button, binding);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if result.success {
                                if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                    let msg = ServerMessage::State { snapshot };
                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        let _ = tx.send(Message::Text(json.into()));
                                    }
                                }
                            }
                        }
                    }

                    ClientMessage::ChooseClass { class } => {
                        if let Some(ref pid) = player_id {
                            let class_id = match game::ClassId::parse(&class) {
                                Some(c) => c,
                                None => {
                                    let resp = ServerMessage::ActionResult {
                                        success: false,
                                        message: "Unknown class.".into(),
                                    };
                                    if let Ok(json) = serde_json::to_string(&resp) {
                                        let _ = tx.send(Message::Text(json.into()));
                                    }
                                    continue;
                                }
                            };
                            let mut game = server.game.write().await;
                            let result = game::choose_class(&mut game, pid, class_id);
                            let success = result.success;
                            let resp = ServerMessage::ActionResult {
                                success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            if success {
                                if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                    let state_msg = ServerMessage::State { snapshot };
                                    if let Ok(json) = serde_json::to_string(&state_msg) {
                                        let _ = tx.send(Message::Text(json.into()));
                                    }
                                }
                            }
                        }
                    }

                    ClientMessage::AllocateSkill { skill_id } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::allocate_skill(&mut game, pid, &skill_id);
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                        }
                    }

                    ClientMessage::CastSkill { skill_id, target_enemy_id, target_x, target_z } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::cast_skill(
                                &mut game,
                                pid,
                                &skill_id,
                                target_enemy_id.as_deref(),
                                target_x,
                                target_z,
                            );
                            let resp = ServerMessage::ActionResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                        }
                    }
                }
            }

            Message::Close(_) => break,
            Message::Ping(data) => {
                let _ = tx.send(Message::Pong(data));
            }
            _ => {}
        }
    }

    // Cleanup on disconnect: keep PlayerState (so the save loop persists final
    // values) but flag it offline so other players no longer see them as online.
    if let Some(ref pid) = player_id {
        info!("Player {} disconnected from {}", pid, addr);
        let mut game = server.game.write().await;
        if let Some(p) = game.players.get_mut(pid) {
            p.online = false;
        }
    }
    {
        let mut clients = server.clients.write().await;
        clients.remove(&conn_id);
    }

    send_task.abort();
    Ok(())
}
