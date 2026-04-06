use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;

use aff_common::error::AppResult;
use aff_core::services::settings_service;

pub async fn get_announcement(
    db: web::Data<DatabaseConnection>,
) -> AppResult<HttpResponse> {
    let enabled = settings_service::get_setting(db.get_ref(), "announcement_enabled")
        .await?
        .unwrap_or_else(|| "false".to_string());

    if enabled != "true" {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "enabled": false,
            "text": "",
            "type": "info",
        })));
    }

    let text = settings_service::get_setting(db.get_ref(), "announcement_text")
        .await?
        .unwrap_or_default();
    let ann_type = settings_service::get_setting(db.get_ref(), "announcement_type")
        .await?
        .unwrap_or_else(|| "info".to_string());

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "enabled": true,
        "text": text,
        "type": ann_type,
    })))
}

pub async fn get_site_info(
    db: web::Data<DatabaseConnection>,
) -> AppResult<HttpResponse> {
    let keys = [
        "site_name",
        "site_description",
        "site_keywords",
        "site_logo",
        "contact_email",
        "contact_telegram",
    ];

    let mut info = serde_json::Map::new();
    for key in &keys {
        let val = settings_service::get_setting(db.get_ref(), key)
            .await?
            .unwrap_or_default();
        info.insert(key.to_string(), serde_json::Value::String(val));
    }

    Ok(HttpResponse::Ok().json(info))
}

pub async fn get_public_config(
    config: web::Data<aff_common::config::AppConfig>,
) -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "allow_command_action": config.security.allow_command_action,
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/announcement", web::get().to(get_announcement))
        .route("/site-info", web::get().to(get_site_info))
        .route("/config", web::get().to(get_public_config));
}
