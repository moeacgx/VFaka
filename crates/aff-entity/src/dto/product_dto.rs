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
