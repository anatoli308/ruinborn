use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::classes::{class_definition, ClassId};
use crate::combat::{maintain_population, tick_enemies, Enemy, LootDrop};
use crate::items::{ActionBar, EquipSlotName, Equipment, ItemBags};
use crate::progression::{starter_progression, Stats};
use crate::skills::tick_player_skill_timers;
use crate::world::{build_default_zones, zone_at, zone_by_id, Zone, ZoneId};

// ── Constants ─────────────────────────────────────────────────

pub const WORLD_BOUND: f64 = 90.0;
pub const GRID_SPACING: f64 = 10.0;
pub const INTERACTION_RANGE: f64 = 5.0;
pub const STARTING_GOLD: f64 = 500.0;
pub const MARKET_CREATION_COST: f64 = 2000.0;
pub const MAX_ACTIVE_MISSIONS: usize = 3;
pub const MISSION_BOARD_SIZE: usize = 8;

/// Snap a world coordinate to the nearest grid point
pub fn snap_to_grid(v: f64) -> f64 {
    (v / GRID_SPACING).round() * GRID_SPACING
}

/// Check if a grid slot is already occupied by a market
fn is_grid_slot_occupied(markets: &[PlayerMarket], gx: f64, gz: f64) -> bool {
    markets.iter().any(|m| (m.x - gx).abs() < 0.1 && (m.z - gz).abs() < 0.1)
}

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

// ── Resource Nodes (REMOVED) ──────────────────────────────────
// Gathering replaced by D2-style enemy kills + loot drops. See combat.rs.

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
    pub show_trade_panel: bool,
    pub trade_history: Vec<TradeRecord>,
    pub notification: String,
    /// 5-Bag-Layout (Backpack + 4 zusätzliche Slots).
    pub bags: ItemBags,
    /// Action Bar mit 9 Slots (Hotkeys 1-9).
    pub action_bar: ActionBar,
    /// D2-style Paperdoll (Helm/Amulet/Chest/.../Ring1/Ring2).
    pub equipment: Equipment,

    // ── D2 progression ──
    pub level: u32,
    pub xp: u64,
    pub xp_to_next: u64,
    pub unspent_stat_points: u32,
    pub stats: Stats,
    pub hp: f64,
    pub max_hp: f64,
    pub mana: f64,
    pub max_mana: f64,
    pub is_dead: bool,
    /// How many ticks until respawn (0 = alive).
    pub respawn_in: u32,

    // ── Zones ──
    pub zone: ZoneId,
    pub unlocked_waypoints: HashSet<ZoneId>,

    // ── Combat input bindings (left/right mouse) ──
    /// What left-click does. None = basic attack.
    pub mouse_left: Option<crate::items::ActionBinding>,
    /// What right-click does. None = basic attack.
    pub mouse_right: Option<crate::items::ActionBinding>,

    // ── Class & skills ──
    /// `None` until the player picks a class on first join.
    pub class_id: Option<ClassId>,
    /// `skill_id -> level` (0..=20). Starter skills are implicitly known at level 1.
    pub allocated_skills: HashMap<String, u32>,
    /// One per level-up, spent via `allocate_skill`.
    pub unspent_skill_points: u32,
    /// `skill_id -> remaining ticks`. Cleaned up automatically when reaching 0.
    pub skill_cooldowns: HashMap<String, u32>,
    /// `buff_id -> remaining ticks` for self-buffs (e.g. battle_cry).
    pub active_buffs: HashMap<String, u32>,

    // ── Damage model (Phase 3) ──
    /// Per-type damage reduction (clamped at 75%).
    #[serde(default)]
    pub resistances: crate::damage::Resistances,
    /// Active damage-over-time effects on the player.
    #[serde(default)]
    pub dots: Vec<crate::damage::DotInstance>,

    // ── Session ──
    /// Transient: true while a WebSocket session for this player is open.
    /// Not persisted (`#[serde(default)]` ⇒ false on reload).
    /// Used to filter snapshots so that logged-out characters do not appear
    /// as "online" to other players.
    #[serde(default, skip_serializing)]
    pub online: bool,
}

// ── Game State ────────────────────────────────────────────────

/// Complete authoritative world state — lives only on the server
#[derive(Debug, Clone)]
pub struct GameState {
    pub tick: u64,
    pub start_time: Instant,
    pub commodities: Vec<Commodity>,
    pub player_markets: Vec<PlayerMarket>,
    pub mission_board: Vec<Mission>,
    pub players: HashMap<String, PlayerState>,

    // ── D2 world ──
    pub zones: Vec<Zone>,
    pub enemies: Vec<Enemy>,
    pub loot_drops: Vec<LootDrop>,
    pub next_enemy_id: u64,
    pub next_loot_id: u64,
}

// ── Snapshots ─────────────────────────────────────────────────

/// Full snapshot sent to a player (on join, after major actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub tick: u64,
    pub elapsed_secs: f64,
    pub player: PlayerState,
    pub other_players: Vec<OtherPlayer>,
    pub commodities: Vec<Commodity>,
    pub player_markets: Vec<PlayerMarket>,
    pub mission_board: Vec<Mission>,
    pub zones: Vec<Zone>,
    pub enemies: Vec<Enemy>,
    pub loot_drops: Vec<LootDrop>,
}

/// Delta snapshot — only contains fields that changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub tick: u64,
    pub elapsed_secs: f64,
    pub player: PlayerState,
    pub other_players: Vec<OtherPlayer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_markets: Option<Vec<PlayerMarket>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mission_board: Option<Vec<Mission>>,
    /// Enemies in this player's current zone (sent every tick).
    pub enemies: Vec<Enemy>,
    /// Loot in this player's current zone.
    pub loot_drops: Vec<LootDrop>,
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
        Commodity { id: "wheat".into(), name: "Wheat".into(), icon: "\u{1F33E}".into(), category: "food".into(), base_value: 10.0 },
        Commodity { id: "iron".into(), name: "Iron".into(), icon: "\u{26CF}\u{FE0F}".into(), category: "material".into(), base_value: 25.0 },
        Commodity { id: "silk".into(), name: "Silk".into(), icon: "\u{1F9F5}".into(), category: "luxury".into(), base_value: 80.0 },
        Commodity { id: "weapons".into(), name: "Weapons".into(), icon: "\u{2694}\u{FE0F}".into(), category: "military".into(), base_value: 120.0 },
        Commodity { id: "spices".into(), name: "Spices".into(), icon: "\u{1F336}\u{FE0F}".into(), category: "food".into(), base_value: 45.0 },
        Commodity { id: "wood".into(), name: "Wood".into(), icon: "\u{1FAB5}".into(), category: "material".into(), base_value: 8.0 },
        Commodity { id: "gems".into(), name: "Gems".into(), icon: "\u{1F48E}".into(), category: "luxury".into(), base_value: 200.0 },
        Commodity { id: "tools".into(), name: "Tools".into(), icon: "\u{1F527}".into(), category: "technology".into(), base_value: 35.0 },
    ];

    let resource_nodes_removed = (); // gathering replaced by enemy combat
    let _ = resource_nodes_removed;

    GameState {
        tick: 0,
        start_time: Instant::now(),
        commodities,
        player_markets: Vec::new(),
        mission_board: Vec::new(),
        players: HashMap::new(),
        zones: build_default_zones(),
        enemies: Vec::new(),
        loot_drops: Vec::new(),
        next_enemy_id: 0,
        next_loot_id: 0,
    }
}

// Resource node generation removed — enemies populate the world now (see combat.rs).

/// Create a new player with starting resources
pub fn create_player(id: &str, name: &str) -> PlayerState {
    let prog = starter_progression();
    let mut unlocked: HashSet<ZoneId> = HashSet::new();
    unlocked.insert(ZoneId::Town);
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
        show_trade_panel: false,
        trade_history: Vec::new(),
        notification: String::new(),
        bags: ItemBags::default(),
        action_bar: ActionBar::default(),
        equipment: Equipment::default(),
        level: prog.level,
        xp: prog.xp,
        xp_to_next: prog.xp_to_next,
        unspent_stat_points: prog.unspent_stat_points,
        stats: prog.stats,
        hp: prog.hp,
        max_hp: prog.max_hp,
        mana: prog.mana,
        max_mana: prog.max_mana,
        is_dead: false,
        respawn_in: 0,
        zone: ZoneId::Town,
        unlocked_waypoints: unlocked,
        mouse_left: None,
        mouse_right: None,
        class_id: None,
        allocated_skills: HashMap::new(),
        unspent_skill_points: 0,
        skill_cooldowns: HashMap::new(),
        active_buffs: HashMap::new(),
        resistances: crate::damage::Resistances::default(),
        dots: Vec::new(),
        online: true,
    }
}

/// First-time class pick. Sets base stats from the class definition and grants
/// the standard 1-per-level skill points the player has earned so far.
pub fn choose_class(state: &mut GameState, player_id: &str, class: ClassId) -> ActionResult {
    let Some(player) = state.players.get_mut(player_id) else {
        return ActionResult { success: false, message: "Spieler nicht gefunden".into() };
    };
    if player.class_id.is_some() {
        return ActionResult { success: false, message: "Klasse bereits gewählt.".into() };
    }
    let def = class_definition(class);
    player.class_id = Some(class);
    player.stats = def.base_stats.clone();
    // Recompute HP/Mana from new base stats; full heal.
    player.max_hp = player.stats.max_hp(player.level);
    player.max_mana = player.stats.max_mana(player.level);
    player.hp = player.max_hp;
    player.mana = player.max_mana;
    // Give 1 unspent skill point per level past 1 (newly created chars stay at 0).
    player.unspent_skill_points = player.level.saturating_sub(1);
    player.notification = format!("🛡️ Klasse gewählt: {}", def.name);
    ActionResult { success: true, message: "OK".into() }
}

// ── Snapshots ─────────────────────────────────────────────────

/// Build a full snapshot for a player (sent on join + after major actions)
pub fn build_player_snapshot(state: &GameState, player_id: &str) -> Option<PlayerSnapshot> {
    let player = state.players.get(player_id)?.clone();
    let other_players = build_other_players(state, player_id);

    Some(PlayerSnapshot {
        tick: state.tick,
        elapsed_secs: state.start_time.elapsed().as_secs_f64(),
        player,
        other_players,
        commodities: state.commodities.clone(),
        player_markets: state.player_markets.clone(),
        mission_board: state.mission_board.clone(),
        zones: state.zones.clone(),
        enemies: state.enemies.clone(),
        loot_drops: state.loot_drops.clone(),
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
    let player_zone = player.zone;
    let enemies: Vec<Enemy> = state.enemies.iter()
        .filter(|e| e.zone == player_zone)
        .cloned()
        .collect();
    let loot_drops: Vec<LootDrop> = state.loot_drops.iter()
        .filter(|l| l.zone == player_zone)
        .cloned()
        .collect();

    Some(DeltaSnapshot {
        tick: state.tick,
        elapsed_secs: state.start_time.elapsed().as_secs_f64(),
        player,
        other_players,
        player_markets: if economy_changed { Some(state.player_markets.clone()) } else { None },
        mission_board: if economy_changed { Some(state.mission_board.clone()) } else { None },
        enemies,
        loot_drops,
    })
}

fn build_other_players(state: &GameState, player_id: &str) -> Vec<OtherPlayer> {
    state.players.iter()
        .filter(|(id, p)| *id != player_id && p.online)
        .map(|(_, p)| OtherPlayer {
            id: p.id.clone(),
            name: p.name.clone(),
            x: p.x,
            z: p.z,
        })
        .collect()
}

// ── Simulation ────────────────────────────────────────────────

/// Advance one economy tick — enemy AI + spawning + missions + cleanup
pub fn advance_tick(state: &mut GameState) {
    let mut rng = rand::thread_rng();

    // Clear per-player notifications
    for player in state.players.values_mut() {
        player.notification.clear();
    }

    // ── Combat tick: feed enemy AI with player positions ──
    let positions: HashMap<String, (ZoneId, f64, f64, bool)> = state.players.iter()
        .map(|(id, p)| (id.clone(), (p.zone, p.x, p.z, !p.is_dead)))
        .collect();
    let hits = tick_enemies(&mut state.enemies, &state.zones, &positions);
    for hit in hits {
        if let Some(player) = state.players.get_mut(&hit.player_id) {
            if player.is_dead { continue; }
            let actual = player.resistances.apply(&hit.damage);
            player.hp -= actual;
            if let Some(dot) = hit.poison_dot {
                // Replace existing poison if new one is at least as strong.
                let new_total = dot.damage_per_tick * dot.ticks_remaining as f64;
                if let Some(existing) = player.dots.iter_mut()
                    .find(|d| d.damage_type == dot.damage_type)
                {
                    let cur = existing.damage_per_tick * existing.ticks_remaining as f64;
                    if new_total >= cur { *existing = dot; }
                } else {
                    player.dots.push(dot);
                }
            }
            if player.hp <= 0.0 {
                player.hp = 0.0;
                player.is_dead = true;
                player.respawn_in = 25; // ~5s @ 5tps
                player.notification = "\u{1F480} Du wurdest get\u{00F6}tet!".into();
            }
        }
    }

    // Tick player DoTs.
    for player in state.players.values_mut() {
        if player.is_dead { continue; }
        let dot_dmg = crate::damage::tick_dots(&mut player.dots, &player.resistances);
        if dot_dmg > 0.0 {
            player.hp -= dot_dmg;
            if player.hp <= 0.0 {
                player.hp = 0.0;
                player.is_dead = true;
                player.respawn_in = 25;
                player.notification = "\u{1F480} Du bist an Gift gestorben!".into();
            }
        }
    }

    // Tick enemy DoTs.
    for enemy in state.enemies.iter_mut() {
        if !enemy.is_alive() { continue; }
        let dot_dmg = crate::damage::tick_dots(&mut enemy.dots, &enemy.resistances);
        if dot_dmg > 0.0 {
            enemy.hp -= dot_dmg;
            if enemy.hp <= 0.0 {
                enemy.hp = 0.0;
                enemy.state = crate::combat::EnemyState::Dead;
                enemy.despawn_in = crate::combat::ENEMY_DESPAWN_TICKS;
            }
        }
    }

    // Player respawn timer.
    let town_spawn = state.zones.iter()
        .find(|z| z.id == ZoneId::Town)
        .map(|z| (z.spawn_x, z.spawn_z))
        .unwrap_or((0.0, 0.0));
    for player in state.players.values_mut() {
        if player.is_dead {
            if player.respawn_in > 0 {
                player.respawn_in -= 1;
            } else {
                player.is_dead = false;
                player.hp = player.max_hp;
                player.mana = player.max_mana;
                player.x = town_spawn.0;
                player.z = town_spawn.1;
                player.zone = ZoneId::Town;
                player.notification = "\u{2728} Du bist in der Stadt wiederbelebt.".into();
            }
        }
    }

    // Tick down skill cooldowns + active buff durations.
    tick_player_skill_timers(state);

    // Maintain enemy population.
    maintain_population(&mut state.enemies, &state.zones, &mut state.next_enemy_id, &mut rng);

    // Loot drops decay after 5 minutes (300 ticks @ 1tps server, but combat at 5tps - approximate).
    let cutoff = state.tick.saturating_sub(300);
    state.loot_drops.retain(|l| l.dropped_tick >= cutoff);

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
            player.notification = format!("\u{23F0} Mission expired: {}", title);
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
    // All missions are sell missions now — gathering removed.
    let qty: u32 = rng.gen_range(2..=8);
    let gold = commodity.base_value * qty as f64 * 0.5;
    let title = format!("Sell {} {}", qty, commodity.name);
    let description = format!("Sell {} units of {} at a player market.", qty, commodity.name);
    let target_quantity = qty;
    let reward_gold = gold;
    let reward_rep = qty * 3;
    let mission_type = "sell";

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
/// Move a player by delta, clamped to world bounds
pub fn move_player(state: &mut GameState, player_id: &str, dx: f64, dz: f64) {
    let market_positions: Vec<(String, f64, f64)> = state.player_markets.iter()
        .map(|m| (m.id.clone(), m.x, m.z))
        .collect();
    let zones_snapshot = state.zones.clone();

    if let Some(player) = state.players.get_mut(player_id) {
        if player.show_trade_panel || player.is_dead { return; }

        let new_x = (player.x + dx).max(-WORLD_BOUND).min(WORLD_BOUND);
        let new_z = (player.z + dz).max(-WORLD_BOUND).min(WORLD_BOUND);

        // Block movement into dead space (between zones).
        if let Some(new_zone) = zone_at(&zones_snapshot, new_x, new_z) {
            player.x = new_x;
            player.z = new_z;
            if new_zone != player.zone {
                player.zone = new_zone;
                player.unlocked_waypoints.insert(new_zone);
                if let Some(zone) = zone_by_id(&zones_snapshot, new_zone) {
                    player.notification = format!("\u{1F5FA}\u{FE0F} Betritt: {}", zone.name);
                }
            }
        }

        player.nearest_market_id = find_nearest_in_range(
            player.x, player.z, INTERACTION_RANGE,
            market_positions.iter().map(|(id, x, z)| (id.as_str(), *x, *z)),
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

/// Gather is removed — kept as no-op for old clients during migration.
pub fn gather_resource(_state: &mut GameState, _player_id: &str) -> ActionResult {
    ActionResult { success: false, message: "Gathering wurde durch Kampf ersetzt. Töte Gegner für Loot.".into() }
}

// ── Item / Bag / ActionBar Mutations ──────────────────────────

/// Move (swap) an item between two bag slots.
pub fn move_item(
    state: &mut GameState,
    player_id: &str,
    src_bag: u32,
    src_slot: u32,
    dst_bag: u32,
    dst_slot: u32,
) -> ActionResult {
    let player = match state.players.get_mut(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };

    let ok = player.bags.swap(src_bag as usize, src_slot as usize, dst_bag as usize, dst_slot as usize);
    if !ok {
        return ActionResult { success: false, message: "\u{274C} Invalid slot.".into() };
    }
    player.action_bar.prune_missing(&player.bags);
    ActionResult { success: true, message: String::new() }
}

/// Drop (delete) an item permanently.
pub fn drop_item(state: &mut GameState, player_id: &str, bag: u32, slot: u32) -> ActionResult {
    let player = match state.players.get_mut(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };
    let removed = player.bags.take(bag as usize, slot as usize);
    if removed.is_none() {
        return ActionResult { success: false, message: "\u{274C} Empty slot.".into() };
    }
    player.action_bar.prune_missing(&player.bags);
    ActionResult { success: true, message: "\u{1F5D1}\u{FE0F} Item dropped.".into() }
}

/// Bind an item to an action-bar slot.
pub fn set_action_slot(
    state: &mut GameState,
    player_id: &str,
    slot: u32,
    item_id: Option<String>,
) -> ActionResult {
    use crate::items::{ActionBinding, ACTION_BAR_SLOTS};

    let player = match state.players.get_mut(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };
    let slot_idx = slot as usize;
    if slot_idx >= ACTION_BAR_SLOTS {
        return ActionResult { success: false, message: "\u{274C} Invalid action slot.".into() };
    }

    match item_id {
        Some(id) => {
            if player.bags.find_position(&id).is_none() {
                return ActionResult { success: false, message: "\u{274C} Item not in bags.".into() };
            }
            player.action_bar.slots[slot_idx] = Some(ActionBinding::Item { item_id: id });
        }
        None => {
            player.action_bar.slots[slot_idx] = None;
        }
    }
    ActionResult { success: true, message: String::new() }
}

/// Bind a *skill* to an action-bar slot (D2-style). The player must already
/// know the skill (allocated points in it OR class-starter).
pub fn set_action_slot_skill(
    state: &mut GameState,
    player_id: &str,
    slot: u32,
    skill_id: String,
) -> ActionResult {
    use crate::items::{ActionBinding, ACTION_BAR_SLOTS};

    let player = match state.players.get_mut(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };
    let slot_idx = slot as usize;
    if slot_idx >= ACTION_BAR_SLOTS {
        return ActionResult { success: false, message: "\u{274C} Ungültiger Slot.".into() };
    }
    if crate::skills::skill_def(&skill_id).is_none() {
        return ActionResult { success: false, message: "\u{274C} Unbekannte Fertigkeit.".into() };
    }
    if !crate::skills::player_knows_skill(player, &skill_id) {
        return ActionResult {
            success: false,
            message: "\u{274C} Du beherrschst diese Fertigkeit nicht.".into(),
        };
    }
    player.action_bar.slots[slot_idx] = Some(ActionBinding::Skill { skill_id });
    ActionResult { success: true, message: String::new() }
}

/// Trigger an action-bar slot. For now: items only show a tooltip notification.
pub fn use_action_slot(state: &mut GameState, player_id: &str, slot: u32) -> ActionResult {
    use crate::items::{ActionBinding, ACTION_BAR_SLOTS};

    let slot_idx = slot as usize;
    if slot_idx >= ACTION_BAR_SLOTS {
        return ActionResult { success: false, message: "\u{274C} Invalid action slot.".into() };
    }

    // Read binding without holding a long mutable borrow (skills need &mut state).
    let binding = match state.players.get(player_id) {
        Some(p) => p.action_bar.slots.get(slot_idx).cloned().flatten(),
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };

    match binding {
        Some(ActionBinding::Item { item_id }) => {
            let player = state.players.get_mut(player_id).unwrap();
            match player.bags.find_position(&item_id) {
                Some((b, s)) => {
                    let it = player.bags.bags[b].as_ref().unwrap().slots[s].as_ref().unwrap();
                    let msg = format!("\u{2728} {} (ilvl {})", it.name, it.item_level);
                    player.notification = msg.clone();
                    ActionResult { success: true, message: msg }
                }
                None => {
                    player.action_bar.slots[slot_idx] = None;
                    ActionResult { success: false, message: "\u{274C} Item gone.".into() }
                }
            }
        }
        Some(ActionBinding::Skill { skill_id }) => {
            // Pick a target appropriate for the skill effect.
            let (px, pz) = match state.players.get(player_id) {
                Some(p) => (p.x, p.z),
                None => return ActionResult { success: false, message: "Player not found!".into() },
            };
            let def = crate::skills::skill_def(&skill_id);
            let needs_enemy = matches!(
                def.as_ref().map(|d| &d.effect),
                Some(crate::skills::SkillEffect::DirectDamage { .. })
                    | Some(crate::skills::SkillEffect::DamageOverTime { .. }),
            );
            let target_enemy_id: Option<String> = if needs_enemy {
                /// Auto-target range for hotbar-cast offensive skills.
                const HOTBAR_TARGET_RANGE: f64 = 18.0;
                let mut best: Option<(String, f64)> = None;
                for e in state.enemies.iter() {
                    if matches!(e.state, crate::combat::EnemyState::Dead) { continue; }
                    let dx = e.x - px;
                    let dz = e.z - pz;
                    let d2 = dx * dx + dz * dz;
                    if d2 > HOTBAR_TARGET_RANGE * HOTBAR_TARGET_RANGE { continue; }
                    if best.as_ref().map_or(true, |(_, bd)| d2 < *bd) {
                        best = Some((e.id.clone(), d2));
                    }
                }
                best.map(|(id, _)| id)
            } else {
                None
            };
            crate::skills::cast_skill(
                state,
                player_id,
                &skill_id,
                target_enemy_id.as_deref(),
                Some(px),
                Some(pz),
            )
        }
        Some(ActionBinding::Attack) => {
            ActionResult { success: false, message: "Nutze Maus, um anzugreifen.".into() }
        }
        None => ActionResult { success: false, message: "\u{274C} Empty slot.".into() },
    }
}

/// Create a player-owned market at the player's current position
pub fn create_market(state: &mut GameState, player_id: &str, name: &str) -> ActionResult {
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };

    if player.owned_market_id.is_some() {
        return ActionResult { success: false, message: "You already own a market!".into() };
    }

    if player.gold < MARKET_CREATION_COST {
        return ActionResult {
            success: false,
            message: format!("\u{274C} Not enough gold! Required: {}", MARKET_CREATION_COST),
        };
    }

    // Snap player position to nearest grid point
    let gx = snap_to_grid(player.x);
    let gz = snap_to_grid(player.z);

    if is_grid_slot_occupied(&state.player_markets, gx, gz) {
        return ActionResult {
            success: false,
            message: "\u{274C} This grid slot is already occupied!".into(),
        };
    }

    let market_id = format!("market_{}_{}", player_id.chars().take(8).collect::<String>(), state.tick);
    let market = PlayerMarket {
        id: market_id.clone(),
        owner_id: player_id.to_string(),
        owner_name: player.name.clone(),
        name: name.to_string(),
        x: gx,
        z: gz,
        orders: Vec::new(),
    };

    state.player_markets.push(market);

    let player = state.players.get_mut(player_id).unwrap();
    player.gold -= MARKET_CREATION_COST;
    player.owned_market_id = Some(market_id);
    player.notification = format!("\u{1F3EA} Market '{}' opened!", name);

    ActionResult { success: true, message: format!("\u{1F3EA} Market '{}' opened!", name) }
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
        return ActionResult { success: false, message: "Invalid quantity!".into() };
    }
    if price_per_unit <= 0.0 {
        return ActionResult { success: false, message: "Invalid price!".into() };
    }
    if !state.commodities.iter().any(|c| c.id == commodity_id) {
        return ActionResult { success: false, message: "Unknown commodity!".into() };
    }

    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };

    let market_id = match &player.owned_market_id {
        Some(id) => id.clone(),
        None => return ActionResult { success: false, message: "You don't own a market!".into() },
    };

    // For sell orders: check inventory
    if order_type == "sell" {
        let owned = player.inventory.get(commodity_id).copied().unwrap_or(0);
        if owned < quantity {
            return ActionResult { success: false, message: "\u{274C} Not enough items in inventory!".into() };
        }
    }

    // For buy orders: check gold
    if order_type == "buy" {
        let total_cost = price_per_unit * quantity as f64;
        if player.gold < total_cost {
            return ActionResult { success: false, message: "\u{274C} Not enough gold for buy order!".into() };
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
        None => return ActionResult { success: false, message: "Market not found!".into() },
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
    let type_label = if order_type == "buy" { "Buy order" } else { "Sell order" };

    ActionResult {
        success: true,
        message: format!("\u{1F4CB} {} placed: {}x {} @ {:.2}g", type_label, quantity, commodity_name, price_per_unit),
    }
}

/// Cancel an order on the player's own market (refund reserved resources)
pub fn cancel_order(state: &mut GameState, player_id: &str, order_id: &str) -> ActionResult {
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };

    let market_id = match &player.owned_market_id {
        Some(id) => id.clone(),
        None => return ActionResult { success: false, message: "You don't own a market!".into() },
    };

    let market = match state.player_markets.iter_mut().find(|m| m.id == market_id) {
        Some(m) => m,
        None => return ActionResult { success: false, message: "Market not found!".into() },
    };

    let order = match market.orders.iter().find(|o| o.id == order_id) {
        Some(o) => o.clone(),
        None => return ActionResult { success: false, message: "Order not found!".into() },
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

    ActionResult { success: true, message: "\u{1F5D1}\u{FE0F} Order cancelled.".into() }
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
        return ActionResult { success: false, message: "Invalid quantity!".into() };
    }

    // Check player is near the market
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };

    if player.nearest_market_id.as_deref() != Some(market_id) {
        return ActionResult { success: false, message: "Not in range of the market!".into() };
    }

    // Can't fill own orders
    if player.owned_market_id.as_deref() == Some(market_id) {
        return ActionResult { success: false, message: "Cannot fill your own orders!".into() };
    }

    // Find order and extract info
    let market = match state.player_markets.iter().find(|m| m.id == market_id) {
        Some(m) => m,
        None => return ActionResult { success: false, message: "Market not found!".into() },
    };

    let order = match market.orders.iter().find(|o| o.id == order_id) {
        Some(o) => o,
        None => return ActionResult { success: false, message: "Order not found!".into() },
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
            return ActionResult { success: false, message: "\u{274C} Not enough gold!".into() };
        }
    } else {
        let owned = player.inventory.get(&commodity_id).copied().unwrap_or(0);
        if owned < fill_qty {
            return ActionResult { success: false, message: "\u{274C} Not enough items!".into() };
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
    let action = if order_type == "sell" { "Bought" } else { "Sold" };

    ActionResult {
        success: true,
        message: format!("\u{2705} {}: {}x {} @ {:.2}g", action, fill_qty, commodity_name, order_price),
    }
}

/// Accept a mission from the board
pub fn accept_mission(state: &mut GameState, player_id: &str, mission_id: &str) -> ActionResult {
    let player = match state.players.get(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Player not found!".into() },
    };

    if player.active_missions.len() >= MAX_ACTIVE_MISSIONS {
        return ActionResult {
            success: false,
            message: format!("\u{274C} Max {} active missions!", MAX_ACTIVE_MISSIONS),
        };
    }

    let mission_idx = match state.mission_board.iter().position(|m| m.id == mission_id) {
        Some(i) => i,
        None => return ActionResult { success: false, message: "Mission not found!".into() },
    };

    let mission = state.mission_board.remove(mission_idx);
    let title = mission.title.clone();

    let player = state.players.get_mut(player_id).unwrap();
    player.active_missions.push(mission);
    player.notification = format!("\u{1F4DC} Mission accepted: {}", title);

    ActionResult { success: true, message: format!("\u{1F4DC} Mission accepted: {}", title) }
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

// ── D2 Combat actions ─────────────────────────────────────────

/// Player attacks an enemy in melee range. Reads stats + equipped weapon for damage.
pub fn player_attack(state: &mut GameState, player_id: &str, enemy_id: &str) -> ActionResult {
    use crate::combat::player_attack_enemy;
    let mut rng = rand::thread_rng();

    let (px, pz, base_damage, level, current_xp, current_xp_to_next) = {
        let Some(p) = state.players.get(player_id) else {
            return ActionResult { success: false, message: "Spieler nicht gefunden".into() };
        };
        if p.is_dead {
            return ActionResult { success: false, message: "Du bist tot.".into() };
        }
        // Weapon damage from main_hand if equipped, else fists.
        let weapon_dmg = p.equipment.weapon.as_ref()
            .map(|it| (it.vendor_value).max(1.0) * 0.05 + 2.0 + (it.item_level as f64) * 0.5)
            .unwrap_or(3.0);
        let total = weapon_dmg + p.stats.melee_bonus();
        (p.x, p.z, total, p.level, p.xp, p.xp_to_next)
    };
    let _ = (level, current_xp, current_xp_to_next);

    let outcome = match player_attack_enemy(
        &mut state.enemies, enemy_id, px, pz, base_damage, state.tick, &mut rng,
    ) {
        Some(o) => o,
        None => return ActionResult { success: false, message: "Ziel ausser Reichweite oder tot.".into() },
    };

    // Spawn loot drop on the ground.
    if let Some(item) = &outcome.loot {
        if let Some(enemy) = state.enemies.iter().find(|e| e.id == enemy_id) {
            state.next_loot_id += 1;
            state.loot_drops.push(crate::combat::LootDrop {
                id: format!("loot_{}", state.next_loot_id),
                item: item.clone(),
                x: enemy.x,
                z: enemy.z,
                zone: enemy.zone,
                dropped_tick: state.tick,
            });
        }
    }

    // Apply XP + gold to player.
    if let Some(p) = state.players.get_mut(player_id) {
        if outcome.killed {
            p.gold += outcome.gold_reward as f64;
            let stats_clone = p.stats.clone();
            let levels = crate::progression::grant_xp(
                &mut p.level, &mut p.xp, &mut p.xp_to_next, &mut p.unspent_stat_points,
                &mut p.hp, &mut p.max_hp, &mut p.mana, &mut p.max_mana,
                &stats_clone, outcome.xp_reward,
            );
            p.unspent_skill_points = p.unspent_skill_points.saturating_add(levels);
            if levels > 0 {
                p.notification = format!(
                    "\u{2B50} Level Up! Stufe {} (+{} Stat-Punkte, +{} Fertigkeitspunkte)",
                    p.level, levels * crate::progression::STAT_POINTS_PER_LEVEL, levels,
                );
            } else {
                p.notification = format!(
                    "\u{2694}\u{FE0F} {} get\u{00F6}tet (+{} XP, +{} Gold)",
                    outcome.enemy_label, outcome.xp_reward, outcome.gold_reward,
                );
            }
        }
    }

    ActionResult {
        success: true,
        message: if outcome.killed { "Kill!".into() } else { format!("{:.0} Schaden", outcome.damage_dealt) },
    }
}

/// Pick up a loot drop on the ground.
pub fn pickup_loot(state: &mut GameState, player_id: &str, loot_id: &str) -> ActionResult {
    use crate::combat::LOOT_PICKUP_RANGE;
    let (px, pz) = match state.players.get(player_id) {
        Some(p) if !p.is_dead => (p.x, p.z),
        _ => return ActionResult { success: false, message: "Du kannst gerade nichts aufheben.".into() },
    };

    let idx = match state.loot_drops.iter().position(|l| l.id == loot_id) {
        Some(i) => i,
        None => return ActionResult { success: false, message: "Loot nicht mehr da.".into() },
    };
    let loot = &state.loot_drops[idx];
    let dx = loot.x - px;
    let dz = loot.z - pz;
    let dist = (dx * dx + dz * dz).sqrt();
    if dist > LOOT_PICKUP_RANGE {
        return ActionResult { success: false, message: "Zu weit weg.".into() };
    }

    let drop = state.loot_drops.remove(idx);
    if let Some(p) = state.players.get_mut(player_id) {
        let item_name = drop.item.name.clone();
        match p.bags.try_add(drop.item.clone()) {
            None => {
                p.notification = format!("\u{1F4E6} Aufgehoben: {}", item_name);
                ActionResult { success: true, message: "OK".into() }
            }
            Some(returned) => {
                // Re-drop the item on the ground.
                state.loot_drops.push(crate::combat::LootDrop {
                    id: drop.id,
                    item: returned,
                    x: drop.x,
                    z: drop.z,
                    zone: drop.zone,
                    dropped_tick: drop.dropped_tick,
                });
                ActionResult { success: false, message: "Inventar voll.".into() }
            }
        }
    } else {
        ActionResult { success: false, message: "Spieler weg.".into() }
    }
}

/// Travel to a known waypoint. Player must already have unlocked the target
/// zone *and* be standing within `WAYPOINT_USE_RANGE` of a waypoint stone in
/// their current zone (D2-style — you can't fast-travel from the middle of a
/// dungeon, you have to walk to the wegpunkt first).
pub fn travel_waypoint(state: &mut GameState, player_id: &str, target: ZoneId) -> ActionResult {
    /// Max distance (units) from the local waypoint stone for travel.
    const WAYPOINT_USE_RANGE: f64 = 6.0;

    let zone = match zone_by_id(&state.zones, target) {
        Some(z) => z.clone(),
        None => return ActionResult { success: false, message: "Zone unbekannt.".into() },
    };
    let Some(player) = state.players.get_mut(player_id) else {
        return ActionResult { success: false, message: "Spieler nicht gefunden".into() };
    };
    if player.is_dead {
        return ActionResult { success: false, message: "Du bist tot.".into() };
    }
    if !player.unlocked_waypoints.contains(&target) {
        return ActionResult { success: false, message: "Wegpunkt nicht entdeckt.".into() };
    }

    // Player must be near the waypoint stone of their *current* zone.
    let here = match zone_by_id(&state.zones, player.zone) {
        Some(z) => z.clone(),
        None => return ActionResult { success: false, message: "Aktuelle Zone unbekannt.".into() },
    };
    let (hx, hz) = match (here.waypoint_x, here.waypoint_z) {
        (Some(x), Some(z)) => (x, z),
        _ => return ActionResult { success: false, message: "Hier gibt es keinen Wegpunkt.".into() },
    };
    let dx = player.x - hx;
    let dz = player.z - hz;
    if (dx * dx + dz * dz).sqrt() > WAYPOINT_USE_RANGE {
        return ActionResult {
            success: false,
            message: "\u{1F300} Geh zum Wegpunkt-Stein, um zu reisen.".into(),
        };
    }

    let (wx, wz) = match (zone.waypoint_x, zone.waypoint_z) {
        (Some(x), Some(z)) => (x, z),
        _ => (zone.spawn_x, zone.spawn_z),
    };
    player.x = wx;
    player.z = wz;
    player.zone = target;
    player.notification = format!("\u{1F300} Reist nach: {}", zone.name);
    ActionResult { success: true, message: "OK".into() }
}

/// Allocate one unspent stat point. `stat` is one of: strength|dexterity|vitality|energy.
pub fn allocate_stat(state: &mut GameState, player_id: &str, stat: &str) -> ActionResult {
    let Some(p) = state.players.get_mut(player_id) else {
        return ActionResult { success: false, message: "Spieler nicht gefunden".into() };
    };
    if p.unspent_stat_points == 0 {
        return ActionResult { success: false, message: "Keine freien Punkte.".into() };
    }
    match stat {
        "strength" => p.stats.strength += 1,
        "dexterity" => p.stats.dexterity += 1,
        "vitality" => {
            p.stats.vitality += 1;
            // Increase max HP, but don't auto-heal.
            let new_max = p.stats.max_hp(p.level);
            let delta = new_max - p.max_hp;
            p.max_hp = new_max;
            p.hp += delta;
        }
        "energy" => {
            p.stats.energy += 1;
            let new_max = p.stats.max_mana(p.level);
            let delta = new_max - p.max_mana;
            p.max_mana = new_max;
            p.mana += delta;
        }
        _ => return ActionResult { success: false, message: "Unbekanntes Attribut.".into() },
    }
    p.unspent_stat_points -= 1;
    p.notification = format!("\u{1F4AA} +1 {}", stat);
    ActionResult { success: true, message: "OK".into() }
}

/// Bind a mouse-button to either basic attack (None) or an item from the bag.
pub fn set_mouse_skill(
    state: &mut GameState,
    player_id: &str,
    button: u8,
    binding: Option<crate::items::ActionBinding>,
) -> ActionResult {
    let Some(p) = state.players.get_mut(player_id) else {
        return ActionResult { success: false, message: "Spieler nicht gefunden".into() };
    };
    match button {
        0 => p.mouse_left = binding,
        1 => p.mouse_right = binding,
        _ => return ActionResult { success: false, message: "Ung\u{00FC}ltige Maustaste".into() },
    }
    ActionResult { success: true, message: "OK".into() }
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
        "\u{1F389} Mission complete: {} (+{:.0}g, +{} Rep)",
        mission.title, mission.reward_gold, mission.reward_reputation
    );

    player.active_missions.retain(|m| m.id != mission_id);
}

// ── Equipment ─────────────────────────────────────────────────

/// Item aus dem Bag in einen Equipment-Slot anlegen.
///
/// `target` = `None` bedeutet: automatischer Default-Slot anhand `item.slot`
/// (Ringe gehen zuerst auf Ring1, sonst Ring2). Liegt schon was im Slot,
/// wird das alte Item zurück in den geleerten Bag-Slot gelegt.
pub fn equip_item(
    state: &mut GameState,
    player_id: &str,
    bag: u32,
    slot: u32,
    target: Option<EquipSlotName>,
) -> ActionResult {
    let player = match state.players.get_mut(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Spieler nicht gefunden.".into() },
    };

    let item = match player.bags.take(bag as usize, slot as usize) {
        Some(it) => it,
        None => return ActionResult { success: false, message: "\u{274C} Slot ist leer.".into() },
    };

    // Welcher Slot?
    let target_slot = match target {
        Some(t) => t,
        None => match player.equipment.default_slot_for(item.slot) {
            Some(t) => t,
            None => {
                // Item zur\u00fcck in den Bag legen.
                let _ = player.bags.try_add(item);
                return ActionResult { success: false, message: "\u{274C} Dieses Item kann nicht angelegt werden.".into() };
            }
        },
    };

    let item_name = item.name.clone();
    match player.equipment.equip(target_slot, item) {
        Ok(prev) => {
            // Vorheriges Item in den Bag zur\u00fcck.
            if let Some(prev_item) = prev {
                let prev_name = prev_item.name.clone();
                if let Some(returned) = player.bags.try_add(prev_item) {
                    // Bag voll \u2014 in den Original-Slot zur\u00fcck.
                    if let Some(b) = player.bags.bags.get_mut(bag as usize).and_then(|b| b.as_mut()) {
                        if let Some(s) = b.slots.get_mut(slot as usize) {
                            *s = Some(returned);
                        }
                    }
                    player.action_bar.prune_missing(&player.bags);
                    return ActionResult {
                        success: false,
                        message: format!("\u{26A0}\u{FE0F} Bags voll \u{2014} [{}] bleibt angelegt.", prev_name),
                    };
                }
            }
            player.action_bar.prune_missing(&player.bags);
            ActionResult {
                success: true,
                message: format!("\u{2705} [{}] angelegt.", item_name),
            }
        }
        Err(returned) => {
            // Typ passt nicht \u2014 Item zur\u00fcck.
            let _ = player.bags.try_add(returned);
            ActionResult {
                success: false,
                message: "\u{274C} Item passt nicht in diesen Slot.".into(),
            }
        }
    }
}

/// Equipment-Slot leeren und das Item zur\u00fcck in den Bag legen.
pub fn unequip_item(
    state: &mut GameState,
    player_id: &str,
    target: EquipSlotName,
) -> ActionResult {
    let player = match state.players.get_mut(player_id) {
        Some(p) => p,
        None => return ActionResult { success: false, message: "Spieler nicht gefunden.".into() },
    };

    let item = match player.equipment.unequip(target) {
        Some(it) => it,
        None => return ActionResult { success: false, message: "\u{274C} Slot ist leer.".into() },
    };

    let item_name = item.name.clone();
    if let Some(returned) = player.bags.try_add(item) {
        // Bag voll \u2014 wieder anlegen.
        let _ = player.equipment.equip(target, returned);
        return ActionResult {
            success: false,
            message: "\u{26A0}\u{FE0F} Bags voll \u{2014} Item bleibt angelegt.".into(),
        };
    }
    ActionResult {
        success: true,
        message: format!("\u{2705} [{}] abgelegt.", item_name),
    }
}