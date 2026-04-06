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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/announcement", web::get().to(get_announcement));
}
