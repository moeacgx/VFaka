use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "withdrawals")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub aff_user_id: i32,
    pub amount: f64,
    pub currency: String,
    pub chain: String,
    pub wallet_address: String,
    pub status: String,
    pub admin_note: Option<String>,
    pub tx_hash: Option<String>,
    pub created_at: DateTimeUtc,
    pub processed_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::aff_user::Entity",
        from = "Column::AffUserId",
        to = "super::aff_user::Column::Id"
    )]
    AffUser,
}

impl Related<super::aff_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AffUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
