use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AffRegisterDto {
    pub email: String,
    pub withdraw_password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AffWithdrawDto {
    pub email: String,
    pub password: String,
    pub amount: f64,
    pub currency: String,
    pub chain: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct AffQueryResponse {
    pub email: String,
    pub aff_code: String,
    pub balance: f64,
    pub total_earned: f64,
    pub total_withdrawn: f64,
    pub level: i32,
    pub level_name: String,
    pub commission_rate: f64,
    pub next_level: Option<AffNextLevel>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AffNextLevel {
    pub level: i32,
    pub name: String,
    pub commission_rate: f64,
    pub required_amount: f64,
    pub remaining: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateAffTierDto {
    pub level: i32,
    pub name: String,
    pub commission_rate: f64,
    pub required_amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAffTierDto {
    pub name: Option<String>,
    pub commission_rate: Option<f64>,
    pub required_amount: Option<f64>,
}
