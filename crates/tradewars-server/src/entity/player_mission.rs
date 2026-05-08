use sea_orm::entity::prelude::*;

/// A mission actively being pursued by a specific player.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "player_missions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub player_id: String,
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
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::player::Entity",
        from = "Column::PlayerId",
        to = "super::player::Column::Id",
        on_delete = "Cascade"
    )]
    Player,
}

impl Related<super::player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Player.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
