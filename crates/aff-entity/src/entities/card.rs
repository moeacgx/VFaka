use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cards")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub product_id: i32,
    pub variant_id: Option<i32>,
    pub content: String,
    pub status: String,
    pub order_id: Option<i32>,
    pub sold_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,
    #[sea_orm(
        belongs_to = "super::product_variant::Entity",
        from = "Column::VariantId",
        to = "super::product_variant::Column::Id"
    )]
    ProductVariant,
    #[sea_orm(
        belongs_to = "super::order::Entity",
        from = "Column::OrderId",
        to = "super::order::Column::Id"
    )]
    Order,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::product_variant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductVariant.def()
    }
}

impl Related<super::order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Order.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
