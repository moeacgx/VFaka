use aff_common::error::{AppError, AppResult};

use crate::epay::{EpayConfig, EpayProvider};
use crate::provider::PaymentProvider;
use crate::tokenpay::{TokenPayConfig, TokenPayProvider};

/// Create a payment provider by channel name and JSON config.
pub fn create_provider(channel: &str, config_json: &str) -> AppResult<Box<dyn PaymentProvider>> {
    match channel {
        "epay" => {
            let config: EpayConfig = serde_json::from_str(config_json)
                .map_err(|e| AppError::BadRequest(format!("Invalid epay config: {}", e)))?;
            Ok(Box::new(EpayProvider::new(config)))
        }
        "tokenpay" => {
            let config: TokenPayConfig = serde_json::from_str(config_json)
                .map_err(|e| AppError::BadRequest(format!("Invalid tokenpay config: {}", e)))?;
            Ok(Box::new(TokenPayProvider::new(config)))
        }
        _ => Err(AppError::BadRequest(format!(
            "Unknown payment channel: {}",
            channel
        ))),
    }
}
