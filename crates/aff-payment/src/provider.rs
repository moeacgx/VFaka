use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use aff_common::types::PaymentMethod;

/// Unified payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub order_no: String,
    pub product_name: String,
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub user_email: String,
    pub client_ip: String,
    pub notify_url: String,
    pub return_url: String,
}

/// Unified payment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub trade_no: String,
    pub pay_url: Option<String>,
    pub qr_code: Option<String>,
}

/// Raw callback data from payment provider
#[derive(Debug, Clone)]
pub struct CallbackRawData {
    pub query_string: Option<String>,
    pub body: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
}

/// Parsed callback data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackData {
    pub order_no: String,
    pub trade_no: String,
    pub amount: f64,
    pub is_success: bool,
    pub pay_time: Option<String>,
    pub raw: serde_json::Value,
}

/// Payment provider trait — implement this for each payment channel
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_payment(&self, req: PaymentRequest) -> Result<PaymentResponse, aff_common::error::AppError>;
    async fn verify_callback(&self, raw: &CallbackRawData) -> Result<CallbackData, aff_common::error::AppError>;
    fn supported_methods(&self) -> Vec<PaymentMethod>;
}
