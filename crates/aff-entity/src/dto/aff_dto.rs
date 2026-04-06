use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AffRegisterDto {
    pub email: String,
    pub withdraw_password: String,
}

#[derive(Debug, Deserialize)]
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
    pub created_at: DateTime<Utc>,
}
