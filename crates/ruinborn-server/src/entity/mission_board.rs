use sea_orm::entity::prelude::*;

/// A mission available on the public mission board.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mission_board")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title: String,
    pub description: String,
    pub mission_type: String,
    pub commodity_id: Option<String>,
    pub target_quantity: i32,
    pub progress: i32,
    pub reward_gold: f64,
    #[sea_orm(column_type = "JsonBinary")]
    pub reward_items: serde_json::Value,
    pub reward_reputation: i32,
    pub expires_tick: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
