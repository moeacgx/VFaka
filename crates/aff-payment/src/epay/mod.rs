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
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
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

    /// Normalize api_url: strip trailing slash and known endpoint suffixes
    fn normalize_base_url(url: &str) -> String {
        let mut base = url.trim().trim_end_matches('/').to_string();
        for suffix in &["/submit.php", "/mapi.php", "/api.php"] {
            if base.ends_with(suffix) {
                base.truncate(base.len() - suffix.len());
                break;
            }
        }
        base
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
struct EpayApiResponse {
    code: Option<i32>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    trade_no: Option<String>,
    #[serde(default)]
    payurl: Option<String>,
    #[serde(default)]
    qrcode: Option<String>,
}

#[async_trait]
impl PaymentProvider for EpayProvider {
    async fn create_payment(&self, req: PaymentRequest) -> AppResult<PaymentResponse> {
        let pay_type = Self::payment_method_to_epay_type(&req.payment_method)?;
        let money = format!("{:.2}", req.amount);
        let base = Self::normalize_base_url(&self.config.api_url);

        let mut params = BTreeMap::new();
        params.insert("pid".to_string(), self.config.pid.clone());
        params.insert("type".to_string(), pay_type.to_string());
        params.insert("out_trade_no".to_string(), req.order_no.clone());
        params.insert("notify_url".to_string(), req.notify_url.clone());
        params.insert("return_url".to_string(), req.return_url.clone());
        params.insert("name".to_string(), req.product_name.clone());
        params.insert("money".to_string(), money);

        let sign_str = build_sign_string(&params);
        let sign = md5_sign(&sign_str, &self.config.key);

        params.insert("sign".to_string(), sign);
        params.insert("sign_type".to_string(), "MD5".to_string());

        // Strategy: try mapi.php API first (server-to-server), fall back to submit.php redirect.
        // mapi.php returns a payurl pointing to the actual payment gateway (alipay, wxpay, etc.),
        // so the user's browser never touches the EPay CDN — avoids region blocking issues.
        let mapi_url = format!("{}/mapi.php", base);
        info!(url = %mapi_url, order_no = %req.order_no, "Trying EPay mapi.php API");

        match self.try_mapi(&mapi_url, &params).await {
            Ok(resp) => {
                info!(order_no = %req.order_no, "EPay mapi.php succeeded");
                return Ok(resp);
            }
            Err(e) => {
                warn!(order_no = %req.order_no, error = %e, "EPay mapi.php failed, falling back to submit.php redirect");
            }
        }

        // Fallback: build submit.php redirect URL for the browser
        let qs: String = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params.iter())
            .finish();
        let pay_url = format!("{}/submit.php?{}", base, qs);

        info!(order_no = %req.order_no, "EPay submit.php fallback URL generated");

        Ok(PaymentResponse {
            trade_no: String::new(),
            pay_url: Some(pay_url),
            qr_code: None,
        })
    }

    async fn verify_callback(&self, raw: &CallbackRawData) -> AppResult<CallbackData> {
        // EPay V1 sends callback as both GET query and POST body
        let data_str = raw
            .query_string
            .as_ref()
            .filter(|s| !s.is_empty())
            .or(raw.body.as_ref())
            .ok_or_else(|| AppError::PaymentError("Missing callback data in epay callback".into()))?;

        let pairs: Vec<(String, String)> = url::form_urlencoded::parse(data_str.as_bytes())
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

impl EpayProvider {
    /// Try mapi.php server-to-server API call.
    /// Returns PaymentResponse with the actual payment gateway URL if successful.
    async fn try_mapi(
        &self,
        mapi_url: &str,
        params: &BTreeMap<String, String>,
    ) -> AppResult<PaymentResponse> {
        let resp = self
            .client
            .get(mapi_url)
            .query(params)
            .send()
            .await
            .map_err(|e| AppError::PaymentError(format!("mapi request failed: {}", e)))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::PaymentError(format!("mapi response read failed: {}", e)))?;

        if body.trim().is_empty() {
            return Err(AppError::PaymentError("mapi returned empty body".into()));
        }

        if !status.is_success() {
            return Err(AppError::PaymentError(format!(
                "mapi returned HTTP {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        let epay_resp: EpayApiResponse = serde_json::from_str(&body).map_err(|e| {
            AppError::PaymentError(format!("mapi parse failed: {} body={}", e, body.chars().take(200).collect::<String>()))
        })?;

        match epay_resp.code {
            Some(1) => {}
            Some(code) => {
                return Err(AppError::PaymentError(format!(
                    "mapi error code={}: {}",
                    code,
                    epay_resp.msg.unwrap_or_default()
                )));
            }
            None => {
                return Err(AppError::PaymentError(format!(
                    "mapi missing code field: {}",
                    body.chars().take(200).collect::<String>()
                )));
            }
        }

        let pay_url = epay_resp
            .payurl
            .filter(|u| !u.is_empty());

        if pay_url.is_none() && epay_resp.qrcode.is_none() {
            return Err(AppError::PaymentError("mapi returned no payurl or qrcode".into()));
        }

        Ok(PaymentResponse {
            trade_no: epay_resp.trade_no.unwrap_or_default(),
            pay_url,
            qr_code: epay_resp.qrcode,
        })
    }
}
