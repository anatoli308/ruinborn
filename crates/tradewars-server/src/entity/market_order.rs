use sea_orm::entity::prelude::*;

/// A buy or sell order posted on a player market.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "market_orders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub market_id: String,
    pub commodity_id: String,
    pub order_type: String,
    pub quantity: i32,
    pub remaining: i32,
    pub price_per_unit: f64,
    pub created_tick: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::player_market::Entity",
        from = "Column::MarketId",
        to = "super::player_market::Column::Id",
        on_delete = "Cascade"
    )]
    PlayerMarket,
}

impl Related<super::player_market::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayerMarket.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
