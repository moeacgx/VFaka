use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "aff_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub aff_user_id: i32,
    pub order_id: i32,
    pub commission: f64,
    pub status: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::aff_user::Entity",
        from = "Column::AffUserId",
        to = "super::aff_user::Column::Id"
    )]
    AffUser,
    #[sea_orm(
        belongs_to = "super::order::Entity",
        from = "Column::OrderId",
        to = "super::order::Column::Id"
    )]
    Order,
}

impl Related<super::aff_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AffUser.def()
    }
}

impl Related<super::order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Order.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
