use std::collections::HashMap;
use std::time::{Duration, Instant};

use log::info;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

use ruinborn_game::{
    GameState, Mission, MarketOrder, PlayerMarket, PlayerState, ResourceNode, TradeRecord,
};

/// Connect to PostgreSQL and return a connection pool.
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    info!("Connected to PostgreSQL");
    Ok(pool)
}

/// Run database migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    info!("Database migrations applied");
    Ok(())
}

/// Load the full game state from the database.
/// Returns `None` if the database has no saved game data (first run after migration).
pub async fn load_game_state(
    pool: &PgPool,
    commodities: Vec<ruinborn_game::Commodity>,
) -> Result<Option<GameState>, sqlx::Error> {
    // Check if any players or resource nodes exist (indicator of a saved game)
    let player_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM players")
        .fetch_one(pool)
        .await?;

    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_nodes")
        .fetch_one(pool)
        .await?;

    if player_count == 0 && node_count == 0 {
        info!("No saved game data found — starting fresh");
        return Ok(None);
    }

    info!("Loading game state from database...");

    // Load game meta
    let meta = sqlx::query("SELECT tick, elapsed_secs FROM game_meta WHERE id = 1")
        .fetch_one(pool)
        .await?;
    let tick: i64 = meta.get("tick");
    let elapsed_secs: f64 = meta.get("elapsed_secs");

    // Reconstruct start_time so that start_time.elapsed() ≈ elapsed_secs
    let start_time = Instant::now()
        .checked_sub(Duration::from_secs_f64(elapsed_secs))
        .unwrap_or_else(Instant::now);

    // Load resource nodes
    let node_rows = sqlx::query(
        "SELECT id, commodity_id, name, x, z, amount, max_amount, respawn_ticks, ticks_until_respawn
         FROM resource_nodes"
    )
    .fetch_all(pool)
    .await?;

    let resource_nodes: Vec<ResourceNode> = node_rows
        .iter()
        .map(|r| ResourceNode {
            id: r.get("id"),
            commodity_id: r.get("commodity_id"),
            name: r.get("name"),
            x: r.get("x"),
            z: r.get("z"),
            amount: r.get::<i32, _>("amount") as u32,
            max_amount: r.get::<i32, _>("max_amount") as u32,
            respawn_ticks: r.get::<i32, _>("respawn_ticks") as u32,
            ticks_until_respawn: r.get::<i32, _>("ticks_until_respawn") as u32,
        })
        .collect();

    // Load player markets + orders
    let market_rows = sqlx::query(
        "SELECT id, owner_id, owner_name, name, x, z FROM player_markets"
    )
    .fetch_all(pool)
    .await?;

    let mut player_markets: Vec<PlayerMarket> = Vec::new();
    for mr in &market_rows {
        let market_id: String = mr.get("id");

        let order_rows = sqlx::query(
            "SELECT id, commodity_id, order_type, quantity, remaining, price_per_unit, created_tick
             FROM market_orders WHERE market_id = $1"
        )
        .bind(&market_id)
        .fetch_all(pool)
        .await?;

        let orders: Vec<MarketOrder> = order_rows
            .iter()
            .map(|o| MarketOrder {
                id: o.get("id"),
                commodity_id: o.get("commodity_id"),
                order_type: o.get("order_type"),
                quantity: o.get::<i32, _>("quantity") as u32,
                remaining: o.get::<i32, _>("remaining") as u32,
                price_per_unit: o.get("price_per_unit"),
                created_tick: o.get::<i64, _>("created_tick") as u64,
            })
            .collect();

        player_markets.push(PlayerMarket {
            id: market_id,
            owner_id: mr.get("owner_id"),
            owner_name: mr.get("owner_name"),
            name: mr.get("name"),
            x: mr.get("x"),
            z: mr.get("z"),
            orders,
        });
    }

    // Load mission board
    let mission_rows = sqlx::query(
        "SELECT id, title, description, mission_type, commodity_id,
                target_quantity, progress, reward_gold, reward_items,
                reward_reputation, expires_tick
         FROM mission_board"
    )
    .fetch_all(pool)
    .await?;

    let mission_board: Vec<Mission> = mission_rows.iter().map(|r| load_mission(r)).collect();

    // Load players
    let player_rows = sqlx::query(
        "SELECT id, name, x, z, gold, inventory, reputation, owned_market_id FROM players"
    )
    .fetch_all(pool)
    .await?;

    let mut players: HashMap<String, PlayerState> = HashMap::new();
    for pr in &player_rows {
        let player_id: String = pr.get("id");
        let inventory_json: serde_json::Value = pr.get("inventory");
        let inventory: HashMap<String, u32> = serde_json::from_value(inventory_json)
            .unwrap_or_default();

        // Load active missions for this player
        let pm_rows = sqlx::query(
            "SELECT id, title, description, mission_type, commodity_id,
                    target_quantity, progress, reward_gold, reward_items,
                    reward_reputation, expires_tick
             FROM player_missions WHERE player_id = $1"
        )
        .bind(&player_id)
        .fetch_all(pool)
        .await?;

        let active_missions: Vec<Mission> = pm_rows.iter().map(|r| load_mission(r)).collect();

        // Load trade history for this player
        let th_rows = sqlx::query(
            "SELECT commodity_id, trade_type, quantity, price_per_unit, market_id, tick
             FROM trade_history WHERE player_id = $1 ORDER BY id"
        )
        .bind(&player_id)
        .fetch_all(pool)
        .await?;

        let trade_history: Vec<TradeRecord> = th_rows
            .iter()
            .map(|r| TradeRecord {
                commodity_id: r.get("commodity_id"),
                trade_type: r.get("trade_type"),
                quantity: r.get::<i32, _>("quantity") as u32,
                price_per_unit: r.get("price_per_unit"),
                market_id: r.get("market_id"),
                tick: r.get::<i64, _>("tick") as u64,
            })
            .collect();

        players.insert(player_id.clone(), PlayerState {
            id: player_id,
            name: pr.get("name"),
            x: pr.get("x"),
            z: pr.get("z"),
            gold: pr.get("gold"),
            inventory,
            reputation: pr.get::<i32, _>("reputation") as u32,
            active_missions,
            owned_market_id: pr.get("owned_market_id"),
            // Transient fields — recomputed each tick
            nearest_market_id: None,
            nearest_node_id: None,
            show_trade_panel: false,
            trade_history,
            notification: String::new(),
        });
    }

    info!(
        "Loaded game state: tick={}, {} players, {} markets, {} nodes",
        tick,
        players.len(),
        player_markets.len(),
        resource_nodes.len()
    );

    Ok(Some(GameState {
        tick: tick as u64,
        start_time,
        commodities,
        player_markets,
        resource_nodes,
        mission_board,
        players,
    }))
}

/// Save the full game state to the database (transactional).
pub async fn save_game_state(pool: &PgPool, state: &GameState) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let elapsed_secs = state.start_time.elapsed().as_secs_f64();

    // Upsert game meta
    sqlx::query(
        "UPDATE game_meta SET tick = $1, elapsed_secs = $2 WHERE id = 1"
    )
    .bind(state.tick as i64)
    .bind(elapsed_secs)
    .execute(&mut *tx)
    .await?;

    // ── Resource nodes (TRUNCATE + INSERT) ──
    sqlx::query("DELETE FROM resource_nodes")
        .execute(&mut *tx)
        .await?;

    for node in &state.resource_nodes {
        sqlx::query(
            "INSERT INTO resource_nodes (id, commodity_id, name, x, z, amount, max_amount, respawn_ticks, ticks_until_respawn)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(&node.id)
        .bind(&node.commodity_id)
        .bind(&node.name)
        .bind(node.x)
        .bind(node.z)
        .bind(node.amount as i32)
        .bind(node.max_amount as i32)
        .bind(node.respawn_ticks as i32)
        .bind(node.ticks_until_respawn as i32)
        .execute(&mut *tx)
        .await?;
    }

    // ── Mission board (TRUNCATE + INSERT) ──
    sqlx::query("DELETE FROM mission_board")
        .execute(&mut *tx)
        .await?;

    for mission in &state.mission_board {
        insert_mission(&mut *tx, "mission_board", None, mission).await?;
    }

    // ── Market orders (delete before markets to satisfy FK) ──
    sqlx::query("DELETE FROM market_orders")
        .execute(&mut *tx)
        .await?;

    // ── Player markets (TRUNCATE + INSERT) ──
    sqlx::query("DELETE FROM player_markets")
        .execute(&mut *tx)
        .await?;

    for market in &state.player_markets {
        sqlx::query(
            "INSERT INTO player_markets (id, owner_id, owner_name, name, x, z)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&market.id)
        .bind(&market.owner_id)
        .bind(&market.owner_name)
        .bind(&market.name)
        .bind(market.x)
        .bind(market.z)
        .execute(&mut *tx)
        .await?;

        for order in &market.orders {
            sqlx::query(
                "INSERT INTO market_orders (id, market_id, commodity_id, order_type, quantity, remaining, price_per_unit, created_tick)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&order.id)
            .bind(&market.id)
            .bind(&order.commodity_id)
            .bind(&order.order_type)
            .bind(order.quantity as i32)
            .bind(order.remaining as i32)
            .bind(order.price_per_unit)
            .bind(order.created_tick as i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    // ── Players ──
    // Delete dependent rows first (FK constraints)
    sqlx::query("DELETE FROM trade_history")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM player_missions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM players")
        .execute(&mut *tx)
        .await?;

    for player in state.players.values() {
        let inventory_json = serde_json::to_value(&player.inventory)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        sqlx::query(
            "INSERT INTO players (id, name, x, z, gold, inventory, reputation, owned_market_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(&player.id)
        .bind(&player.name)
        .bind(player.x)
        .bind(player.z)
        .bind(player.gold)
        .bind(&inventory_json)
        .bind(player.reputation as i32)
        .bind(&player.owned_market_id)
        .execute(&mut *tx)
        .await?;

        // Active missions
        for mission in &player.active_missions {
            insert_mission(&mut *tx, "player_missions", Some(&player.id), mission).await?;
        }

        // Trade history
        for trade in &player.trade_history {
            sqlx::query(
                "INSERT INTO trade_history (player_id, commodity_id, trade_type, quantity, price_per_unit, market_id, tick)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(&player.id)
            .bind(&trade.commodity_id)
            .bind(&trade.trade_type)
            .bind(trade.quantity as i32)
            .bind(trade.price_per_unit)
            .bind(&trade.market_id)
            .bind(trade.tick as i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────

/// Parse a mission row from any mission table.
fn load_mission(r: &sqlx::postgres::PgRow) -> Mission {
    let reward_items_json: serde_json::Value = r.get("reward_items");
    let reward_items: HashMap<String, u32> = serde_json::from_value(reward_items_json)
        .unwrap_or_default();

    Mission {
        id: r.get("id"),
        title: r.get("title"),
        description: r.get("description"),
        mission_type: r.get("mission_type"),
        commodity_id: r.get("commodity_id"),
        target_quantity: r.get::<i32, _>("target_quantity") as u32,
        progress: r.get::<i32, _>("progress") as u32,
        reward_gold: r.get("reward_gold"),
        reward_items,
        reward_reputation: r.get::<i32, _>("reward_reputation") as u32,
        expires_tick: r.get::<i64, _>("expires_tick") as u64,
    }
}

/// Insert a mission into mission_board or player_missions.
async fn insert_mission(
    tx: &mut sqlx::PgConnection,
    table: &str,
    player_id: Option<&str>,
    mission: &Mission,
) -> Result<(), sqlx::Error> {
    let reward_items_json = serde_json::to_value(&mission.reward_items)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    if let Some(pid) = player_id {
        sqlx::query(&format!(
            "INSERT INTO {} (id, player_id, title, description, mission_type, commodity_id, target_quantity, progress, reward_gold, reward_items, reward_reputation, expires_tick)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)", table
        ))
        .bind(&mission.id)
        .bind(pid)
        .bind(&mission.title)
        .bind(&mission.description)
        .bind(&mission.mission_type)
        .bind(&mission.commodity_id)
        .bind(mission.target_quantity as i32)
        .bind(mission.progress as i32)
        .bind(mission.reward_gold)
        .bind(&reward_items_json)
        .bind(mission.reward_reputation as i32)
        .bind(mission.expires_tick as i64)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(&format!(
            "INSERT INTO {} (id, title, description, mission_type, commodity_id, target_quantity, progress, reward_gold, reward_items, reward_reputation, expires_tick)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)", table
        ))
        .bind(&mission.id)
        .bind(&mission.title)
        .bind(&mission.description)
        .bind(&mission.mission_type)
        .bind(&mission.commodity_id)
        .bind(mission.target_quantity as i32)
        .bind(mission.progress as i32)
        .bind(mission.reward_gold)
        .bind(&reward_items_json)
        .bind(mission.reward_reputation as i32)
        .bind(mission.expires_tick as i64)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}
