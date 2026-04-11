use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateVariantDto {
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariantDto {
    pub name: Option<String>,
    pub price: Option<f64>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct VariantResponse {
    pub id: i32,
    pub product_id: i32,
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub stock_count: i32,
    pub sales_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
