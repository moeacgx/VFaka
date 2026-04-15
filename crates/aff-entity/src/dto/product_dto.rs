use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::variant_dto::VariantResponse;

#[derive(Debug, Deserialize)]
pub struct CreateProductDto {
    pub category_id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub is_active: Option<bool>,
    pub allow_alipay: Option<bool>,
    pub allow_wxpay: Option<bool>,
    pub allow_qqpay: Option<bool>,
    pub allow_usdt_trc20: Option<bool>,
    pub allow_trx: Option<bool>,
    pub allow_usdt_erc20: Option<bool>,
    pub post_pay_action_type: Option<String>,
    pub post_pay_action_value: Option<String>,
    pub delivery_mode: Option<String>,
    pub aff_commission_rate: Option<f64>,
    pub sort_order: Option<i32>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductDto {
    pub category_id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub is_active: Option<bool>,
    pub allow_alipay: Option<bool>,
    pub allow_wxpay: Option<bool>,
    pub allow_qqpay: Option<bool>,
    pub allow_usdt_trc20: Option<bool>,
    pub allow_trx: Option<bool>,
    pub allow_usdt_erc20: Option<bool>,
    pub post_pay_action_type: Option<String>,
    pub post_pay_action_value: Option<String>,
    pub delivery_mode: Option<String>,
    pub aff_commission_rate: Option<f64>,
    pub sort_order: Option<i32>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: i32,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub variants: Vec<VariantResponse>,
}

/// Public-facing product response — strips internal fields (post_pay_action, aff_commission)
#[derive(Debug, Serialize)]
pub struct PublicProductResponse {
    pub id: i32,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
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
    pub delivery_mode: String,
    pub sort_order: i32,
    pub min_quantity: i32,
    pub max_quantity: i32,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub variants: Vec<VariantResponse>,
}

impl From<ProductResponse> for PublicProductResponse {
    fn from(p: ProductResponse) -> Self {
        Self {
            id: p.id,
            category_id: p.category_id,
            category_name: p.category_name,
            name: p.name,
            description: p.description,
            price: p.price,
            stock_count: p.stock_count,
            sales_count: p.sales_count,
            is_active: p.is_active,
            allow_alipay: p.allow_alipay,
            allow_wxpay: p.allow_wxpay,
            allow_qqpay: p.allow_qqpay,
            allow_usdt_trc20: p.allow_usdt_trc20,
            allow_trx: p.allow_trx,
            allow_usdt_erc20: p.allow_usdt_erc20,
            delivery_mode: p.delivery_mode,
            sort_order: p.sort_order,
            min_quantity: p.min_quantity,
            max_quantity: p.max_quantity,
            image_url: p.image_url,
            video_url: p.video_url,
            variants: p.variants,
        }
    }
}
