use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateCouponDto {
    pub code: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub product_id: Option<i32>,
    pub min_amount: Option<f64>,
    pub max_uses: Option<i32>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCouponDto {
    pub code: Option<String>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub product_id: Option<Option<i32>>,
    pub min_amount: Option<f64>,
    pub max_uses: Option<Option<i32>>,
    pub valid_from: Option<Option<DateTime<Utc>>>,
    pub valid_to: Option<Option<DateTime<Utc>>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateCouponDto {
    pub code: String,
    pub product_id: i32,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct ValidateCouponResponse {
    pub valid: bool,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub discount_amount: Option<f64>,
    pub message: Option<String>,
}
