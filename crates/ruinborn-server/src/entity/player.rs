use sea_orm::entity::prelude::*;

/// Per-player persistent state.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "players")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
    pub gold: f64,
    #[sea_orm(column_type = "JsonBinary")]
    pub inventory: serde_json::Value,
    pub reputation: i32,
    pub owned_market_id: Option<String>,
    /// JSONB: serialisierte `ItemBags` (5 Bag-Slots, Default-Backpack).
    #[sea_orm(column_type = "JsonBinary")]
    pub bags: serde_json::Value,
    /// JSONB: serialisierte `ActionBar` (9 Hotkey-Slots).
    #[sea_orm(column_type = "JsonBinary")]
    pub action_bar: serde_json::Value,
    /// JSONB: serialisierte `Equipment` (D2-Paperdoll, 10 Slots).
    #[sea_orm(column_type = "JsonBinary")]
    pub equipment: serde_json::Value,

    // ── D2 progression ──
    pub level: i32,
    pub xp: i64,
    pub xp_to_next: i64,
    pub unspent_stat_points: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub stats: serde_json::Value,
    pub hp: f64,
    pub max_hp: f64,
    pub mana: f64,
    pub max_mana: f64,

    // ── Zone state ──
    pub zone: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub unlocked_waypoints: serde_json::Value,

    // ── Mouse skill bindings (nullable) ──
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub mouse_left: Option<serde_json::Value>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub mouse_right: Option<serde_json::Value>,

    // ── Class & skills ──
    /// snake_case class id ("barbarian"|"sorceress"|"necromancer") or NULL until chosen.
    #[sea_orm(nullable)]
    pub class_id: Option<String>,
    /// JSONB: HashMap<String, u32> of allocated skill levels.
    #[sea_orm(column_type = "JsonBinary")]
    pub allocated_skills: serde_json::Value,
    pub unspent_skill_points: i32,
    /// JSONB: HashMap<String, u32> remaining-tick map for skill cooldowns.
    #[sea_orm(column_type = "JsonBinary")]
    pub skill_cooldowns: serde_json::Value,
    /// JSONB: HashMap<String, u32> remaining-tick map for self-buffs.
    #[sea_orm(column_type = "JsonBinary")]
    pub active_buffs: serde_json::Value,

    // ── Damage model (Phase 3) ──
    /// JSONB: serialised `Resistances` struct.
    #[sea_orm(column_type = "JsonBinary")]
    pub resistances: serde_json::Value,
    /// JSONB: array of `DotInstance`.
    #[sea_orm(column_type = "JsonBinary")]
    pub dots: serde_json::Value,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::player_mission::Entity")]
    PlayerMissions,
    #[sea_orm(has_many = "super::trade_history::Entity")]
    TradeHistory,
}

impl Related<super::player_mission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayerMissions.def()
    }
}

impl Related<super::trade_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TradeHistory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
