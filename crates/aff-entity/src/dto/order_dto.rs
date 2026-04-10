use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateOrderDto {
    pub product_id: i32,
    pub quantity: i32,
    pub email: String,
    pub payment_method: String,
    pub aff_code: Option<String>,
    pub coupon_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderQueryDto {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: i32,
    pub order_no: String,
    pub product_id: i32,
    pub product_name: Option<String>,
    pub quantity: i32,
    pub total_amount: f64,
    pub email: String,
    pub payment_method: String,
    pub payment_channel: String,
    pub status: String,
    pub trade_no: Option<String>,
    pub pay_time: Option<DateTime<Utc>>,
    pub aff_code: Option<String>,
    pub cards_snapshot: Option<String>,
    pub post_action_result: Option<String>,
    pub post_action_status: Option<String>,
    pub coupon_code: Option<String>,
    pub discount_amount: f64,
    pub query_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
