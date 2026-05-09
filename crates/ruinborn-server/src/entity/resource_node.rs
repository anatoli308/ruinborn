use sea_orm::entity::prelude::*;

/// A gatherable resource node on the world map.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "resource_nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub commodity_id: String,
    pub name: String,
    pub x: f64,
    pub z: f64,
    pub amount: i32,
    pub max_amount: i32,
    pub respawn_ticks: i32,
    pub ticks_until_respawn: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
