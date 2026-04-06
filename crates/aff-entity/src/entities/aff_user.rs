use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "aff_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique)]
    pub aff_code: String,
    pub balance: f64,
    pub total_earned: f64,
    pub total_withdrawn: f64,
    #[serde(skip_serializing)]
    pub withdraw_password_hash: Option<String>,
    pub level: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::aff_log::Entity")]
    AffLogs,
    #[sea_orm(has_many = "super::withdrawal::Entity")]
    Withdrawals,
}

impl Related<super::aff_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AffLogs.def()
    }
}

impl Related<super::withdrawal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Withdrawals.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
