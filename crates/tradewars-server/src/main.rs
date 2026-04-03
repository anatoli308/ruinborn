use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;

use tradewars_game::{self as game, GameState};
use tradewars_protocol::{ClientMessage, ServerMessage};

/// Server configuration
const BIND_ADDR: &str = "0.0.0.0:9000";
const TICK_RATE: u64 = 1; // 1 tick per second (economy sim)

/// Per-connection sender handle
type Tx = mpsc::UnboundedSender<Message>;

/// Shared server state
struct Server {
    game: RwLock<GameState>,
    clients: RwLock<HashMap<String, ClientHandle>>,
}

struct ClientHandle {
    tx: Tx,
    player_id: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let server = Arc::new(Server {
        game: RwLock::new(game::create_initial_state()),
        clients: RwLock::new(HashMap::new()),
    });

    // Start the tick loop
    let tick_server = Arc::clone(&server);
    tokio::spawn(async move {
        tick_loop(tick_server).await;
    });

    // Bind TCP listener
    let listener = TcpListener::bind(BIND_ADDR).await.expect("Failed to bind");
    info!("TradeWars server listening on {}", BIND_ADDR);

    while let Ok((stream, addr)) = listener.accept().await {
        let server = Arc::clone(&server);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(server, stream, addr).await {
                error!("Connection error from {}: {}", addr, e);
            }
        });
    }
}

// ── Tick Loop ─────────────────────────────────────────────────

/// World simulation runs forever — no pause, no speed control (MMO-style)
async fn tick_loop(server: Arc<Server>) {
    let mut interval = tokio::time::interval(Duration::from_secs(TICK_RATE));

    loop {
        interval.tick().await;

        // Advance simulation
        {
            let mut game = server.game.write().await;
            game::advance_tick(&mut game);
        }

        // Broadcast per-player snapshots
        broadcast_state(&server).await;
    }
}

/// Send each connected player their personalized snapshot
async fn broadcast_state(server: &Server) {
    let game = server.game.read().await;
    let clients = server.clients.read().await;

    for (_, client) in clients.iter() {
        if let Some(snapshot) = game::build_player_snapshot(&game, &client.player_id) {
            let msg = ServerMessage::State { snapshot };
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
                            let new_player = game::create_player(&pid, &name);
                            game.players.insert(pid.clone(), new_player);
                        }

                        // Register client handle
                        {
                            let mut clients = server.clients.write().await;
                            clients.insert(conn_id.clone(), ClientHandle {
                                tx: tx.clone(),
                                player_id: pid.clone(),
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
                            // Send immediate response for smooth movement
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = tx.send(Message::Text(json.into()));
                                }
                            }
                        }
                    }

                    ClientMessage::Trade { commodity_id, trade_type, quantity } => {
                        if let Some(ref pid) = player_id {
                            let mut game = server.game.write().await;
                            let result = game::execute_trade(&mut game, pid, &commodity_id, &trade_type, quantity);
                            let resp = ServerMessage::TradeResult {
                                success: result.success,
                                message: result.message,
                            };
                            if let Ok(json) = serde_json::to_string(&resp) {
                                let _ = tx.send(Message::Text(json.into()));
                            }
                            // Also send updated state
                            if let Some(snapshot) = game::build_player_snapshot(&game, pid) {
                                let state_msg = ServerMessage::State { snapshot };
                                if let Ok(json) = serde_json::to_string(&state_msg) {
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
                }
            }

            Message::Close(_) => break,
            Message::Ping(data) => {
                let _ = tx.send(Message::Pong(data));
            }
            _ => {}
        }
    }

    // Cleanup on disconnect
    if let Some(ref pid) = player_id {
        info!("Player {} disconnected from {}", pid, addr);
        let mut game = server.game.write().await;
        game.players.remove(pid);
    }
    {
        let mut clients = server.clients.write().await;
        clients.remove(&conn_id);
    }

    send_task.abort();
    Ok(())
}
