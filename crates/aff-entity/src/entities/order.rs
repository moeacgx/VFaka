use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub order_no: String,
    pub product_id: i32,
    pub quantity: i32,
    pub total_amount: f64,
    pub email: String,
    pub payment_method: String,
    pub payment_channel: String,
    pub status: String,
    pub trade_no: Option<String>,
    pub pay_time: Option<DateTimeUtc>,
    pub aff_code: Option<String>,
    pub aff_user_email: Option<String>,
    pub aff_commission: f64,
    pub cards_snapshot: Option<String>,
    pub post_action_result: Option<String>,
    pub ip_address: Option<String>,
    pub coupon_code: Option<String>,
    pub discount_amount: f64,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,
    #[sea_orm(has_many = "super::card::Entity")]
    Cards,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::card::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cards.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
