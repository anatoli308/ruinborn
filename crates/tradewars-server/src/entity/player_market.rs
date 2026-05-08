use sea_orm::entity::prelude::*;

/// A player-owned market stall placed in the world.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "player_markets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub owner_id: String,
    pub owner_name: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::market_order::Entity")]
    MarketOrders,
}

impl Related<super::market_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MarketOrders.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
