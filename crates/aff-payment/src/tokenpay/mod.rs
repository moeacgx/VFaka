use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use aff_common::crypto::md5_sign::{tokenpay_sign, tokenpay_verify};
use aff_common::error::{AppError, AppResult};
use aff_common::types::PaymentMethod;

use crate::provider::{CallbackData, CallbackRawData, PaymentProvider, PaymentRequest, PaymentResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayConfig {
    pub api_url: String,
    pub notify_secret: String,
    #[serde(default)]
    pub custom_domain: String,
}

pub struct TokenPayProvider {
    config: TokenPayConfig,
    client: reqwest::Client,
}

impl TokenPayProvider {
    pub fn new(config: TokenPayConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn rewrite_pay_url(&self, original_url: &str) -> String {
        let custom_domain = self.config.custom_domain.trim().trim_end_matches('/');
        if custom_domain.is_empty() {
            return original_url.to_string();
        }

        let parsed = match reqwest::Url::parse(original_url) {
            Ok(value) => value,
            Err(_) => return original_url.to_string(),
        };

        let mut rewritten = format!("{}{}", custom_domain, parsed.path());
        if let Some(query) = parsed.query() {
            rewritten.push('?');
            rewritten.push_str(query);
        }
        rewritten
    }

    fn payment_method_to_currency(method: &PaymentMethod) -> AppResult<&'static str> {
        match method {
            PaymentMethod::UsdtTrc20 => Ok("USDT_TRC20"),
            PaymentMethod::Trx => Ok("TRX"),
            PaymentMethod::UsdtErc20 => Ok("USDT_ERC20"),
            PaymentMethod::UsdcErc20 => Ok("USDC_ERC20"),
            _ => Err(AppError::PaymentError(format!(
                "Unsupported payment method for TokenPay: {:?}",
                method
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenPayCreateResponse {
    code: Option<i32>,
    msg: Option<String>,
    data: Option<TokenPayCreateData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct TokenPayCreateData {
    order_id: Option<String>,
    pay_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct TokenPayCallbackBody {
    block_transaction_id: Option<String>,
    out_order_id: Option<String>,
    order_user_key: Option<String>,
    amount: Option<f64>,
    actual_amount: Option<f64>,
    currency: Option<String>,
    status: Option<i32>,
    signature: Option<String>,
}

#[async_trait]
impl PaymentProvider for TokenPayProvider {
    async fn create_payment(&self, req: PaymentRequest) -> AppResult<PaymentResponse> {
        let currency = Self::payment_method_to_currency(&req.payment_method)?;
        let actual_amount = format!("{:.2}", req.amount);

        let mut params = BTreeMap::new();
        params.insert("OutOrderId".to_string(), req.order_no.clone());
        params.insert("OrderUserKey".to_string(), req.user_email.clone());
        params.insert("ActualAmount".to_string(), actual_amount.clone());
        params.insert("Currency".to_string(), currency.to_string());
        params.insert("NotifyUrl".to_string(), req.notify_url.clone());
        params.insert("RedirectUrl".to_string(), req.return_url.clone());

        let signature = tokenpay_sign(&params, &self.config.notify_secret);
        params.insert("Signature".to_string(), signature);

        let api_url = format!("{}/CreateOrder", self.config.api_url.trim_end_matches('/'));
        info!(url = %api_url, order_no = %req.order_no, "Creating TokenPay order");

        let resp = self
            .client
            .post(&api_url)
            .json(&params)
            .send()
            .await
            .map_err(|e| AppError::PaymentError(format!("TokenPay request failed: {}", e)))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::PaymentError(format!("TokenPay response read failed: {}", e)))?;

        if !status.is_success() {
            return Err(AppError::PaymentError(format!(
                "TokenPay API returned status {}: {}",
                status, body
            )));
        }

        let tp_resp: TokenPayCreateResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::PaymentError(format!("TokenPay response parse failed: {} body={}", e, body)))?;

        if tp_resp.code.unwrap_or(-1) != 0 {
            return Err(AppError::PaymentError(format!(
                "TokenPay create order failed: {}",
                tp_resp.msg.unwrap_or_default()
            )));
        }

        let data = tp_resp.data.unwrap_or(TokenPayCreateData {
            order_id: None,
            pay_url: None,
        });

        match &data.pay_url {
            Some(url) if !url.is_empty() => {}
            _ => {
                return Err(AppError::PaymentError(
                    "TokenPay returned no pay_url".to_string(),
                ));
            }
        }

        let pay_url = data
            .pay_url
            .as_deref()
            .map(|url| self.rewrite_pay_url(url));

        Ok(PaymentResponse {
            trade_no: data.order_id.unwrap_or_default(),
            pay_url,
            qr_code: None,
        })
    }

    async fn verify_callback(&self, raw: &CallbackRawData) -> AppResult<CallbackData> {
        let body_str = raw
            .body
            .as_ref()
            .ok_or_else(|| AppError::PaymentError("Missing body in TokenPay callback".into()))?;

        let cb: TokenPayCallbackBody = serde_json::from_str(body_str)
            .map_err(|e| AppError::PaymentError(format!("TokenPay callback parse failed: {}", e)))?;

        let signature = cb
            .signature
            .as_ref()
            .ok_or_else(|| AppError::PaymentError("Missing Signature in TokenPay callback".into()))?;

        // Rebuild params for verification (exclude Signature)
        let mut params = BTreeMap::new();
        if let Some(v) = &cb.block_transaction_id {
            params.insert("BlockTransactionId".to_string(), v.clone());
        }
        if let Some(v) = &cb.out_order_id {
            params.insert("OutOrderId".to_string(), v.clone());
        }
        if let Some(v) = &cb.order_user_key {
            params.insert("OrderUserKey".to_string(), v.clone());
        }
        if let Some(v) = cb.amount {
            params.insert("Amount".to_string(), format!("{}", v));
        }
        if let Some(v) = cb.actual_amount {
            params.insert("ActualAmount".to_string(), format!("{}", v));
        }
        if let Some(v) = &cb.currency {
            params.insert("Currency".to_string(), v.clone());
        }
        if let Some(v) = cb.status {
            params.insert("Status".to_string(), v.to_string());
        }

        if !tokenpay_verify(&params, signature, &self.config.notify_secret) {
            warn!("TokenPay callback signature verification failed");
            return Err(AppError::PaymentError("Invalid signature".into()));
        }

        let is_success = cb.status == Some(1);
        let order_no = cb.out_order_id.clone().unwrap_or_default();
        let trade_no = cb.block_transaction_id.clone().unwrap_or_default();
        let amount = cb.actual_amount.or(cb.amount).unwrap_or(0.0);

        let raw_json: serde_json::Value = serde_json::from_str(body_str).unwrap_or_default();

        info!(order_no = %order_no, trade_no = %trade_no, is_success, "TokenPay callback verified");

        Ok(CallbackData {
            order_no,
            trade_no,
            amount,
            is_success,
            pay_time: None,
            raw: raw_json,
        })
    }

    fn supported_methods(&self) -> Vec<PaymentMethod> {
        vec![
            PaymentMethod::UsdtTrc20,
            PaymentMethod::Trx,
            PaymentMethod::UsdtErc20,
            PaymentMethod::UsdcErc20,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_pay_url_uses_custom_domain() {
        let provider = TokenPayProvider::new(TokenPayConfig {
            api_url: "http://tokenpay:5000".to_string(),
            notify_secret: "secret".to_string(),
            custom_domain: "https://pay.example.com".to_string(),
        });

        let actual = provider.rewrite_pay_url("http://tokenpay:5000/OrderInfo?id=123&currency=TRX");
        assert_eq!(actual, "https://pay.example.com/OrderInfo?id=123&currency=TRX");
    }

    #[test]
    fn test_rewrite_pay_url_keeps_original_when_custom_domain_empty() {
        let provider = TokenPayProvider::new(TokenPayConfig {
            api_url: "http://tokenpay:5000".to_string(),
            notify_secret: "secret".to_string(),
            custom_domain: "".to_string(),
        });

        let actual = provider.rewrite_pay_url("http://tokenpay:5000/OrderInfo?id=123");
        assert_eq!(actual, "http://tokenpay:5000/OrderInfo?id=123");
    }
}
