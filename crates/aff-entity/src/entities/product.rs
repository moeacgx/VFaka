use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub category_id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock_count: i32,
    pub sales_count: i32,
    pub is_active: bool,
    pub allow_alipay: bool,
    pub allow_wxpay: bool,
    pub allow_qqpay: bool,
    pub allow_usdt_trc20: bool,
    pub allow_trx: bool,
    pub allow_usdt_erc20: bool,
    pub post_pay_action_type: Option<String>,
    pub post_pay_action_value: Option<String>,
    pub delivery_mode: String,
    pub aff_commission_rate: Option<f64>,
    pub sort_order: i32,
    pub min_quantity: i32,
    pub max_quantity: i32,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::CategoryId",
        to = "super::category::Column::Id"
    )]
    Category,
    #[sea_orm(has_many = "super::card::Entity")]
    Cards,
    #[sea_orm(has_many = "super::order::Entity")]
    Orders,
    #[sea_orm(has_many = "super::product_variant::Entity")]
    Variants,
}

impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::card::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cards.def()
    }
}

impl Related<super::order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl Related<super::product_variant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Variants.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
