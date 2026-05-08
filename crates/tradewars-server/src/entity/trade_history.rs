use sea_orm::entity::prelude::*;

/// Record of a completed trade, linked to a player.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "trade_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub player_id: String,
    pub commodity_id: String,
    pub trade_type: String,
    pub quantity: i32,
    pub price_per_unit: f64,
    pub market_id: String,
    pub tick: i64,
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
