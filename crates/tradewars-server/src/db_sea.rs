use std::collections::HashMap;
use std::time::{Duration, Instant};

use log::info;
use sea_orm::sea_query::TableCreateStatement;
use sea_orm::*;

use tradewars_game::{
    ActionBar, Equipment, GameState, ItemBags, MarketOrder, Mission, PlayerMarket, PlayerState,
    Stats, TradeRecord, ZoneId,
};

use crate::entity;

// ── Connection ────────────────────────────────────────────────

/// Connect to PostgreSQL via SeaORM.
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    info!("Connected to PostgreSQL via SeaORM");
    Ok(db)
}

// ── Schema Setup ──────────────────────────────────────────────

/// Create all tables from entity definitions (idempotent).
pub async fn ensure_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    // Order matters — referenced tables first
    create_table_if_missing(db, builder, schema.create_table_from_entity(entity::game_meta::Entity)).await?;
    create_table_if_missing(db, builder, schema.create_table_from_entity(entity::player::Entity)).await?;
    create_table_if_missing(db, builder, schema.create_table_from_entity(entity::player_market::Entity)).await?;
    create_table_if_missing(db, builder, schema.create_table_from_entity(entity::market_order::Entity)).await?;
    create_table_if_missing(db, builder, schema.create_table_from_entity(entity::mission_board::Entity)).await?;
    create_table_if_missing(db, builder, schema.create_table_from_entity(entity::player_mission::Entity)).await?;
    create_table_if_missing(db, builder, schema.create_table_from_entity(entity::trade_history::Entity)).await?;

    // Idempotente Migration für Items/Bags/ActionBar (Postgres ADD COLUMN IF NOT EXISTS).
    db.execute_unprepared(
        "ALTER TABLE players \
         ADD COLUMN IF NOT EXISTS bags JSONB NOT NULL DEFAULT '{\"bags\":[null,null,null,null,null]}'::jsonb, \
         ADD COLUMN IF NOT EXISTS action_bar JSONB NOT NULL DEFAULT '{\"slots\":[null,null,null,null,null,null,null,null,null]}'::jsonb, \
         ADD COLUMN IF NOT EXISTS equipment JSONB NOT NULL DEFAULT '{}'::jsonb, \
         ADD COLUMN IF NOT EXISTS level INT NOT NULL DEFAULT 1, \
         ADD COLUMN IF NOT EXISTS xp BIGINT NOT NULL DEFAULT 0, \
         ADD COLUMN IF NOT EXISTS xp_to_next BIGINT NOT NULL DEFAULT 100, \
         ADD COLUMN IF NOT EXISTS unspent_stat_points INT NOT NULL DEFAULT 0, \
         ADD COLUMN IF NOT EXISTS stats JSONB NOT NULL DEFAULT '{\"strength\":10,\"dexterity\":10,\"vitality\":10,\"energy\":10}'::jsonb, \
         ADD COLUMN IF NOT EXISTS hp DOUBLE PRECISION NOT NULL DEFAULT 70.0, \
         ADD COLUMN IF NOT EXISTS max_hp DOUBLE PRECISION NOT NULL DEFAULT 70.0, \
         ADD COLUMN IF NOT EXISTS mana DOUBLE PRECISION NOT NULL DEFAULT 30.0, \
         ADD COLUMN IF NOT EXISTS max_mana DOUBLE PRECISION NOT NULL DEFAULT 30.0, \
         ADD COLUMN IF NOT EXISTS zone TEXT NOT NULL DEFAULT 'town', \
         ADD COLUMN IF NOT EXISTS unlocked_waypoints JSONB NOT NULL DEFAULT '[\"town\"]'::jsonb, \
         ADD COLUMN IF NOT EXISTS mouse_left JSONB, \
         ADD COLUMN IF NOT EXISTS mouse_right JSONB, \
         ADD COLUMN IF NOT EXISTS class_id TEXT, \
         ADD COLUMN IF NOT EXISTS allocated_skills JSONB NOT NULL DEFAULT '{}'::jsonb, \
         ADD COLUMN IF NOT EXISTS unspent_skill_points INT NOT NULL DEFAULT 0, \
         ADD COLUMN IF NOT EXISTS skill_cooldowns JSONB NOT NULL DEFAULT '{}'::jsonb, \
         ADD COLUMN IF NOT EXISTS active_buffs JSONB NOT NULL DEFAULT '{}'::jsonb, \
         ADD COLUMN IF NOT EXISTS resistances JSONB NOT NULL DEFAULT '{}'::jsonb, \
         ADD COLUMN IF NOT EXISTS dots JSONB NOT NULL DEFAULT '[]'::jsonb",
    )
    .await?;

    // Seed singleton game_meta row
    let count = entity::game_meta::Entity::find().count(db).await?;
    if count == 0 {
        entity::game_meta::ActiveModel {
            id: Set(1),
            tick: Set(0),
            elapsed_secs: Set(0.0),
        }
        .insert(db)
        .await?;
    }

    info!("Database schema ensured");
    Ok(())
}

/// Execute a CREATE TABLE IF NOT EXISTS statement.
async fn create_table_if_missing(
    db: &DatabaseConnection,
    builder: DbBackend,
    mut stmt: TableCreateStatement,
) -> Result<(), DbErr> {
    stmt.if_not_exists();
    db.execute(builder.build(&stmt)).await?;
    Ok(())
}

// ── Load ──────────────────────────────────────────────────────

/// Load the full game state from the database.
/// Returns `None` if the database has no saved game data (first run).
pub async fn load_game_state(
    db: &DatabaseConnection,
    commodities: Vec<tradewars_game::Commodity>,
) -> Result<Option<GameState>, DbErr> {
    let player_count = entity::player::Entity::find().count(db).await?;
    let node_count = entity::resource_node::Entity::find().count(db).await?;

    if player_count == 0 && node_count == 0 {
        info!("No saved game data found — starting fresh");
        return Ok(None);
    }

    info!("Loading game state from database...");

    // ── Game meta ──
    let meta = entity::game_meta::Entity::find_by_id(1)
        .one(db)
        .await?
        .expect("game_meta row must exist");

    let start_time = Instant::now()
        .checked_sub(Duration::from_secs_f64(meta.elapsed_secs))
        .unwrap_or_else(Instant::now);

    // ── Player markets + orders ──
    let market_models = entity::player_market::Entity::find().all(db).await?;
    let mut player_markets: Vec<PlayerMarket> = Vec::with_capacity(market_models.len());

    for mm in market_models {
        let order_models = entity::market_order::Entity::find()
            .filter(entity::market_order::Column::MarketId.eq(&mm.id))
            .all(db)
            .await?;

        let orders: Vec<MarketOrder> = order_models.into_iter().map(to_market_order).collect();

        player_markets.push(PlayerMarket {
            id: mm.id,
            owner_id: mm.owner_id,
            owner_name: mm.owner_name,
            name: mm.name,
            x: mm.x,
            z: mm.z,
            orders,
        });
    }

    // ── Mission board ──
    let mission_models = entity::mission_board::Entity::find().all(db).await?;
    let mission_board: Vec<Mission> = mission_models.into_iter().map(to_board_mission).collect();

    // ── Players ──
    let player_models = entity::player::Entity::find().all(db).await?;
    let mut players: HashMap<String, PlayerState> = HashMap::new();

    for pm in player_models {
        let inventory: HashMap<String, u32> =
            serde_json::from_value(pm.inventory).unwrap_or_default();
        let bags: ItemBags =
            serde_json::from_value(pm.bags).unwrap_or_default();
        let action_bar: ActionBar =
            serde_json::from_value(pm.action_bar).unwrap_or_default();
        let equipment: Equipment =
            serde_json::from_value(pm.equipment).unwrap_or_default();
        let stats: Stats =
            serde_json::from_value(pm.stats).unwrap_or_default();
        let unlocked_waypoints: std::collections::HashSet<ZoneId> =
            serde_json::from_value(pm.unlocked_waypoints).unwrap_or_else(|_| {
                let mut s = std::collections::HashSet::new();
                s.insert(ZoneId::Town);
                s
            });
        let zone: ZoneId = match pm.zone.as_str() {
            "wilderness" => ZoneId::Wilderness,
            "burial_grounds" => ZoneId::BurialGrounds,
            _ => ZoneId::Town,
        };
        let mouse_left = pm.mouse_left.and_then(|v| serde_json::from_value(v).ok());
        let mouse_right = pm.mouse_right.and_then(|v| serde_json::from_value(v).ok());
        let class_id = pm.class_id.as_deref().and_then(tradewars_game::ClassId::parse);
        let allocated_skills: std::collections::HashMap<String, u32> =
            serde_json::from_value(pm.allocated_skills).unwrap_or_default();
        let skill_cooldowns: std::collections::HashMap<String, u32> =
            serde_json::from_value(pm.skill_cooldowns).unwrap_or_default();
        let active_buffs: std::collections::HashMap<String, u32> =
            serde_json::from_value(pm.active_buffs).unwrap_or_default();
        let resistances: tradewars_game::Resistances =
            serde_json::from_value(pm.resistances).unwrap_or_default();
        let dots: Vec<tradewars_game::DotInstance> =
            serde_json::from_value(pm.dots).unwrap_or_default();

        let mission_models = entity::player_mission::Entity::find()
            .filter(entity::player_mission::Column::PlayerId.eq(&pm.id))
            .all(db)
            .await?;
        let active_missions: Vec<Mission> =
            mission_models.into_iter().map(to_player_mission).collect();

        let trade_models = entity::trade_history::Entity::find()
            .filter(entity::trade_history::Column::PlayerId.eq(&pm.id))
            .all(db)
            .await?;
        let trade_history: Vec<TradeRecord> =
            trade_models.into_iter().map(to_trade_record).collect();

        players.insert(
            pm.id.clone(),
            PlayerState {
                id: pm.id,
                name: pm.name,
                x: pm.x,
                z: pm.z,
                gold: pm.gold,
                inventory,
                reputation: pm.reputation as u32,
                active_missions,
                owned_market_id: pm.owned_market_id,
                nearest_market_id: None,
                bags,
                action_bar,
                equipment,
                show_trade_panel: false,
                trade_history,
                notification: String::new(),
                level: pm.level.max(1) as u32,
                xp: pm.xp.max(0) as u64,
                xp_to_next: pm.xp_to_next.max(1) as u64,
                unspent_stat_points: pm.unspent_stat_points.max(0) as u32,
                stats,
                hp: pm.hp,
                max_hp: pm.max_hp,
                mana: pm.mana,
                max_mana: pm.max_mana,
                is_dead: false,
                respawn_in: 0,
                zone,
                unlocked_waypoints,
                mouse_left,
                mouse_right,
                class_id,
                allocated_skills,
                unspent_skill_points: pm.unspent_skill_points.max(0) as u32,
                skill_cooldowns,
                active_buffs,
                resistances,
                dots,
                online: false,
            },
        );
    }

    info!(
        "Loaded game state: tick={}, {} players, {} markets",
        meta.tick,
        players.len(),
        player_markets.len(),
    );

    Ok(Some(GameState {
        tick: meta.tick as u64,
        start_time,
        commodities,
        player_markets,
        mission_board,
        players,
        zones: tradewars_game::build_default_zones(),
        enemies: Vec::new(),
        loot_drops: Vec::new(),
        next_enemy_id: 0,
        next_loot_id: 0,
    }))
}

// ── Save ──────────────────────────────────────────────────────

/// Save the full game state to the database (transactional).
pub async fn save_game_state(
    db: &DatabaseConnection,
    state: &GameState,
) -> Result<(), DbErr> {
    let elapsed_secs = state.start_time.elapsed().as_secs_f64();

    db.transaction::<_, (), DbErr>(|txn| {
        // Move everything into the async block via owned copies
        let tick = state.tick as i64;
        let mission_board = state.mission_board.clone();
        let player_markets = state.player_markets.clone();
        let players: Vec<PlayerState> = state.players.values().cloned().collect();

        Box::pin(async move {
            // ── Game meta ──
            entity::game_meta::ActiveModel {
                id: Set(1),
                tick: Set(tick),
                elapsed_secs: Set(elapsed_secs),
            }
            .update(txn)
            .await?;

            // ── Mission board ──
            entity::mission_board::Entity::delete_many().exec(txn).await?;
            for mission in &mission_board {
                insert_board_mission(txn, mission).await?;
            }

            // ── Market orders (delete before markets — FK) ──
            entity::market_order::Entity::delete_many().exec(txn).await?;
            entity::player_market::Entity::delete_many().exec(txn).await?;

            for market in &player_markets {
                entity::player_market::ActiveModel {
                    id: Set(market.id.clone()),
                    owner_id: Set(market.owner_id.clone()),
                    owner_name: Set(market.owner_name.clone()),
                    name: Set(market.name.clone()),
                    x: Set(market.x),
                    z: Set(market.z),
                }
                .insert(txn)
                .await?;

                for order in &market.orders {
                    entity::market_order::ActiveModel {
                        id: Set(order.id.clone()),
                        market_id: Set(market.id.clone()),
                        commodity_id: Set(order.commodity_id.clone()),
                        order_type: Set(order.order_type.clone()),
                        quantity: Set(order.quantity as i32),
                        remaining: Set(order.remaining as i32),
                        price_per_unit: Set(order.price_per_unit),
                        created_tick: Set(order.created_tick as i64),
                    }
                    .insert(txn)
                    .await?;
                }
            }

            // ── Players (delete dependents first — FK) ──
            entity::trade_history::Entity::delete_many().exec(txn).await?;
            entity::player_mission::Entity::delete_many().exec(txn).await?;
            entity::player::Entity::delete_many().exec(txn).await?;

            for player in &players {
                let inventory_json = serde_json::to_value(&player.inventory)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let bags_json = serde_json::to_value(&player.bags)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let action_bar_json = serde_json::to_value(&player.action_bar)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let equipment_json = serde_json::to_value(&player.equipment)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let stats_json = serde_json::to_value(&player.stats)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let unlocked_json = serde_json::to_value(&player.unlocked_waypoints)
                    .unwrap_or(serde_json::Value::Array(Vec::new()));
                let zone_str = match player.zone {
                    ZoneId::Town => "town",
                    ZoneId::Wilderness => "wilderness",
                    ZoneId::BurialGrounds => "burial_grounds",
                };
                let mouse_left_json = player.mouse_left.as_ref().and_then(|b| serde_json::to_value(b).ok());
                let mouse_right_json = player.mouse_right.as_ref().and_then(|b| serde_json::to_value(b).ok());
                let class_id_str = player.class_id.map(|c| match c {
                    tradewars_game::ClassId::Barbarian => "barbarian".to_string(),
                    tradewars_game::ClassId::Sorceress => "sorceress".to_string(),
                    tradewars_game::ClassId::Necromancer => "necromancer".to_string(),
                });
                let allocated_skills_json = serde_json::to_value(&player.allocated_skills)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let skill_cooldowns_json = serde_json::to_value(&player.skill_cooldowns)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let active_buffs_json = serde_json::to_value(&player.active_buffs)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let resistances_json = serde_json::to_value(&player.resistances)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let dots_json = serde_json::to_value(&player.dots)
                    .unwrap_or(serde_json::Value::Array(Vec::new()));

                entity::player::ActiveModel {
                    id: Set(player.id.clone()),
                    name: Set(player.name.clone()),
                    x: Set(player.x),
                    z: Set(player.z),
                    gold: Set(player.gold),
                    inventory: Set(inventory_json),
                    reputation: Set(player.reputation as i32),
                    owned_market_id: Set(player.owned_market_id.clone()),
                    bags: Set(bags_json),
                    action_bar: Set(action_bar_json),
                    equipment: Set(equipment_json),
                    level: Set(player.level as i32),
                    xp: Set(player.xp as i64),
                    xp_to_next: Set(player.xp_to_next as i64),
                    unspent_stat_points: Set(player.unspent_stat_points as i32),
                    stats: Set(stats_json),
                    hp: Set(player.hp),
                    max_hp: Set(player.max_hp),
                    mana: Set(player.mana),
                    max_mana: Set(player.max_mana),
                    zone: Set(zone_str.to_string()),
                    unlocked_waypoints: Set(unlocked_json),
                    mouse_left: Set(mouse_left_json),
                    mouse_right: Set(mouse_right_json),
                    class_id: Set(class_id_str),
                    allocated_skills: Set(allocated_skills_json),
                    unspent_skill_points: Set(player.unspent_skill_points as i32),
                    skill_cooldowns: Set(skill_cooldowns_json),
                    active_buffs: Set(active_buffs_json),
                    resistances: Set(resistances_json),
                    dots: Set(dots_json),
                }
                .insert(txn)
                .await?;

                for mission in &player.active_missions {
                    insert_player_mission(txn, &player.id, mission).await?;
                }

                for trade in &player.trade_history {
                    entity::trade_history::ActiveModel {
                        id: NotSet,
                        player_id: Set(player.id.clone()),
                        commodity_id: Set(trade.commodity_id.clone()),
                        trade_type: Set(trade.trade_type.clone()),
                        quantity: Set(trade.quantity as i32),
                        price_per_unit: Set(trade.price_per_unit),
                        market_id: Set(trade.market_id.clone()),
                        tick: Set(trade.tick as i64),
                    }
                    .insert(txn)
                    .await?;
                }
            }

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        sea_orm::TransactionError::Connection(db_err) => db_err,
        sea_orm::TransactionError::Transaction(db_err) => db_err,
    })?;

    Ok(())
}

// ── Conversion Helpers (DB model → game type) ─────────────────


fn to_market_order(m: entity::market_order::Model) -> MarketOrder {
    MarketOrder {
        id: m.id,
        commodity_id: m.commodity_id,
        order_type: m.order_type,
        quantity: m.quantity as u32,
        remaining: m.remaining as u32,
        price_per_unit: m.price_per_unit,
        created_tick: m.created_tick as u64,
    }
}

fn to_board_mission(m: entity::mission_board::Model) -> Mission {
    Mission {
        id: m.id,
        title: m.title,
        description: m.description,
        mission_type: m.mission_type,
        commodity_id: m.commodity_id,
        target_quantity: m.target_quantity as u32,
        progress: m.progress as u32,
        reward_gold: m.reward_gold,
        reward_items: serde_json::from_value(m.reward_items).unwrap_or_default(),
        reward_reputation: m.reward_reputation as u32,
        expires_tick: m.expires_tick as u64,
    }
}

fn to_player_mission(m: entity::player_mission::Model) -> Mission {
    Mission {
        id: m.id,
        title: m.title,
        description: m.description,
        mission_type: m.mission_type,
        commodity_id: m.commodity_id,
        target_quantity: m.target_quantity as u32,
        progress: m.progress as u32,
        reward_gold: m.reward_gold,
        reward_items: serde_json::from_value(m.reward_items).unwrap_or_default(),
        reward_reputation: m.reward_reputation as u32,
        expires_tick: m.expires_tick as u64,
    }
}

fn to_trade_record(m: entity::trade_history::Model) -> TradeRecord {
    TradeRecord {
        commodity_id: m.commodity_id,
        trade_type: m.trade_type,
        quantity: m.quantity as u32,
        price_per_unit: m.price_per_unit,
        market_id: m.market_id,
        tick: m.tick as u64,
    }
}

// ── Insert Helpers (game type → DB model) ─────────────────────

async fn insert_board_mission<C: ConnectionTrait>(
    db: &C,
    mission: &Mission,
) -> Result<(), DbErr> {
    let reward_items_json = serde_json::to_value(&mission.reward_items)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    entity::mission_board::ActiveModel {
        id: Set(mission.id.clone()),
        title: Set(mission.title.clone()),
        description: Set(mission.description.clone()),
        mission_type: Set(mission.mission_type.clone()),
        commodity_id: Set(mission.commodity_id.clone()),
        target_quantity: Set(mission.target_quantity as i32),
        progress: Set(mission.progress as i32),
        reward_gold: Set(mission.reward_gold),
        reward_items: Set(reward_items_json),
        reward_reputation: Set(mission.reward_reputation as i32),
        expires_tick: Set(mission.expires_tick as i64),
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn insert_player_mission<C: ConnectionTrait>(
    db: &C,
    player_id: &str,
    mission: &Mission,
) -> Result<(), DbErr> {
    let reward_items_json = serde_json::to_value(&mission.reward_items)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    entity::player_mission::ActiveModel {
        id: Set(mission.id.clone()),
        player_id: Set(player_id.to_string()),
        title: Set(mission.title.clone()),
        description: Set(mission.description.clone()),
        mission_type: Set(mission.mission_type.clone()),
        commodity_id: Set(mission.commodity_id.clone()),
        target_quantity: Set(mission.target_quantity as i32),
        progress: Set(mission.progress as i32),
        reward_gold: Set(mission.reward_gold),
        reward_items: Set(reward_items_json),
        reward_reputation: Set(mission.reward_reputation as i32),
        expires_tick: Set(mission.expires_tick as i64),
    }
    .insert(db)
    .await?;

    Ok(())
}
