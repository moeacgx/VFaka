use std::collections::BTreeMap;

use async_trait::async_trait;
use md5;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use aff_common::error::{AppError, AppResult};
use aff_common::types::PaymentMethod;

use crate::provider::{CallbackData, CallbackRawData, PaymentProvider, PaymentRequest, PaymentResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpayConfig {
    pub pid: String,
    pub key: String,
    pub api_url: String,
}

pub struct EpayProvider {
    config: EpayConfig,
    client: reqwest::Client,
}

impl EpayProvider {
    pub fn new(config: EpayConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn payment_method_to_epay_type(method: &PaymentMethod) -> AppResult<&'static str> {
        match method {
            PaymentMethod::Alipay => Ok("alipay"),
            PaymentMethod::Wxpay => Ok("wxpay"),
            PaymentMethod::Qqpay => Ok("qqpay"),
            _ => Err(AppError::PaymentError(format!(
                "Unsupported payment method for epay: {:?}",
                method
            ))),
        }
    }
}

/// Build sign string: sort params by ASCII key, join as key=value&..., exclude sign/sign_type/empty
fn build_sign_string(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .filter(|(k, v)| !v.is_empty() && *k != "sign" && *k != "sign_type")
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}

/// MD5 sign: sign = md5(sign_string + key)
fn md5_sign(sign_str: &str, key: &str) -> String {
    let input = format!("{}{}", sign_str, key);
    format!("{:x}", md5::compute(input.as_bytes()))
}

/// MD5 verify: recompute and compare
fn md5_verify(sign_str: &str, sign_value: &str, key: &str) -> bool {
    let expected = md5_sign(sign_str, key);
    expected == sign_value.to_lowercase()
}

#[derive(Debug, Deserialize)]
struct EpayCreateResponse {
    code: i32,
    msg: Option<String>,
    trade_no: Option<String>,
    payurl: Option<String>,
    qrcode: Option<String>,
}

#[async_trait]
impl PaymentProvider for EpayProvider {
    async fn create_payment(&self, req: PaymentRequest) -> AppResult<PaymentResponse> {
        let pay_type = Self::payment_method_to_epay_type(&req.payment_method)?;
        let money = format!("{:.2}", req.amount);

        let mut params = BTreeMap::new();
        params.insert("pid".to_string(), self.config.pid.clone());
        params.insert("type".to_string(), pay_type.to_string());
        params.insert("out_trade_no".to_string(), req.order_no.clone());
        params.insert("notify_url".to_string(), req.notify_url.clone());
        params.insert("return_url".to_string(), req.return_url.clone());
        params.insert("name".to_string(), req.product_name.clone());
        params.insert("money".to_string(), money);
        params.insert("clientip".to_string(), req.client_ip.clone());

        let sign_str = build_sign_string(&params);
        let sign = md5_sign(&sign_str, &self.config.key);

        params.insert("sign".to_string(), sign);
        params.insert("sign_type".to_string(), "MD5".to_string());

        let api_url = format!("{}/mapi.php", self.config.api_url.trim_end_matches('/'));
        info!(url = %api_url, order_no = %req.order_no, "Creating epay order");

        let resp = self
            .client
            .post(&api_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::PaymentError(format!("Epay request failed: {}", e)))?;

        let body = resp
            .text()
            .await
            .map_err(|e| AppError::PaymentError(format!("Epay response read failed: {}", e)))?;

        let epay_resp: EpayCreateResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::PaymentError(format!("Epay response parse failed: {} body={}", e, body)))?;

        if epay_resp.code != 1 {
            return Err(AppError::PaymentError(format!(
                "Epay create order failed: {}",
                epay_resp.msg.unwrap_or_default()
            )));
        }

        Ok(PaymentResponse {
            trade_no: epay_resp.trade_no.unwrap_or_default(),
            pay_url: epay_resp.payurl,
            qr_code: epay_resp.qrcode,
        })
    }

    async fn verify_callback(&self, raw: &CallbackRawData) -> AppResult<CallbackData> {
        let qs = raw
            .query_string
            .as_ref()
            .ok_or_else(|| AppError::PaymentError("Missing query string in epay callback".into()))?;

        let pairs: Vec<(String, String)> = url::form_urlencoded::parse(qs.as_bytes())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let mut params = BTreeMap::new();
        let mut sign_value = String::new();

        for (k, v) in &pairs {
            if k == "sign" {
                sign_value = v.clone();
            } else if k != "sign_type" {
                params.insert(k.clone(), v.clone());
            }
        }

        if sign_value.is_empty() {
            return Err(AppError::PaymentError("Missing sign in epay callback".into()));
        }

        let sign_str = build_sign_string(&params);
        if !md5_verify(&sign_str, &sign_value, &self.config.key) {
            warn!("Epay callback signature verification failed");
            return Err(AppError::PaymentError("Invalid signature".into()));
        }

        let trade_status = params.get("trade_status").map(|s| s.as_str()).unwrap_or("");
        let is_success = trade_status == "TRADE_SUCCESS";

        let order_no = params
            .get("out_trade_no")
            .cloned()
            .unwrap_or_default();
        let trade_no = params
            .get("trade_no")
            .cloned()
            .unwrap_or_default();
        let amount: f64 = params
            .get("money")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let raw_json = serde_json::to_value(&params).unwrap_or_default();

        info!(order_no = %order_no, trade_no = %trade_no, is_success, "Epay callback verified");

        Ok(CallbackData {
            order_no,
            trade_no,
            amount,
            is_success,
            pay_time: params.get("success_at").cloned(),
            raw: raw_json,
        })
    }

    fn supported_methods(&self) -> Vec<PaymentMethod> {
        vec![PaymentMethod::Alipay, PaymentMethod::Wxpay, PaymentMethod::Qqpay]
    }
}
