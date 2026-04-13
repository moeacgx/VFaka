use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::{AppError, AppResult};
use aff_core::services::payment_config_service;

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentConfigDto {
    pub config_json: String,
    pub is_active: bool,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/payment-configs")
        .route("", web::get().to(list))
        .route("/{channel}", web::put().to(update))
        .route("/{channel}/test", web::post().to(test_connection))
}

fn mask_sensitive_value(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 6 {
        "*".repeat(chars.len())
    } else {
        let prefix: String = chars[..3].iter().collect();
        let suffix: String = chars[chars.len()-3..].iter().collect();
        format!("{}***{}", prefix, suffix)
    }
}

fn mask_config_json(json_str: &str) -> String {
    let sensitive_keys = ["key", "secret", "api_key", "api_secret", "merchant_key", "password", "token"];
    if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(map) = obj.as_object_mut() {
            for (k, v) in map.iter_mut() {
                let k_lower = k.to_lowercase();
                if sensitive_keys.iter().any(|sk| k_lower.contains(sk)) {
                    if let Some(s) = v.as_str() {
                        *v = serde_json::Value::String(mask_sensitive_value(s));
                    }
                }
            }
        }
        serde_json::to_string(&obj).unwrap_or_else(|_| json_str.to_string())
    } else {
        json_str.to_string()
    }
}

async fn list(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let configs = payment_config_service::list_configs(&db).await?;
    let masked: Vec<serde_json::Value> = configs.iter().map(|c| {
        serde_json::json!({
            "id": c.id,
            "channel": c.channel,
            "config_json": mask_config_json(&c.config_json),
            "is_active": c.is_active,
            "created_at": c.created_at,
            "updated_at": c.updated_at,
        })
    }).collect();
    Ok(HttpResponse::Ok().json(masked))
}

async fn update(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    body: web::Json<UpdatePaymentConfigDto>,
) -> AppResult<HttpResponse> {
    let channel = path.into_inner();
    let dto = body.into_inner();

    // Merge masked values with stored secrets to prevent overwrite
    let merged_json = merge_masked_config(&db, &channel, &dto.config_json).await?;

    let config =
        payment_config_service::update_config(&db, &channel, &merged_json, dto.is_active)
            .await?;
    Ok(HttpResponse::Ok().json(config))
}

async fn test_connection(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let channel = path.into_inner();
    let configs = payment_config_service::list_configs(&db).await?;
    let config = configs
        .iter()
        .find(|c| c.channel == channel)
        .ok_or_else(|| AppError::NotFound(format!("Payment config not found: {}", channel)))?;

    let config_obj: serde_json::Value = serde_json::from_str(&config.config_json)
        .map_err(|e| AppError::BadRequest(format!("Invalid config: {}", e)))?;

    let api_url = config_obj
        .get("api_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing api_url in config".into()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    match client.get(api_url).send().await {
        Ok(resp) => {
            let status = resp.status();
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": status.is_success() || status.is_redirection(),
                "message": format!("HTTP {}", status),
            })))
        }
        Err(e) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "message": format!("Connection failed: {}", e),
        }))),
    }
}

/// If the incoming config_json contains masked values (***), replace them
/// with the real values from the stored config. This prevents admins from
/// accidentally overwriting secrets when they only toggle is_active.
async fn merge_masked_config(
    db: &DatabaseConnection,
    channel: &str,
    new_json: &str,
) -> AppResult<String> {
    let new_obj: serde_json::Value = serde_json::from_str(new_json)
        .map_err(|e| AppError::BadRequest(format!("Invalid config JSON: {}", e)))?;

    let configs = payment_config_service::list_configs(db).await?;
    let existing = configs.iter().find(|c| c.channel == channel);

    let Some(existing) = existing else {
        return Ok(new_json.to_string());
    };

    let old_obj: serde_json::Value = serde_json::from_str(&existing.config_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let mut merged = new_obj;
    if let (Some(new_map), Some(old_map)) = (merged.as_object_mut(), old_obj.as_object()) {
        for (key, value) in new_map.iter_mut() {
            if let Some(s) = value.as_str() {
                if s.contains("***") {
                    if let Some(old_val) = old_map.get(key) {
                        *value = old_val.clone();
                    }
                }
            }
        }
    }

    serde_json::to_string(&merged)
        .map_err(|e| AppError::Internal(format!("Failed to serialize config: {}", e)))
}
