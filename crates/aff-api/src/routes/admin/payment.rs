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
}

fn mask_sensitive_value(value: &str) -> String {
    if value.len() <= 6 {
        "*".repeat(value.len())
    } else {
        format!("{}***{}", &value[..3], &value[value.len()-3..])
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
