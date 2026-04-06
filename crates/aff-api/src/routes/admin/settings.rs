use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::collections::HashMap;

use aff_common::error::AppResult;
use aff_core::services::settings_service;

#[derive(Debug, Deserialize)]
pub struct TestNotifyDto {
    pub target: Option<String>,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/settings")
        .route("", web::get().to(get_all))
        .route("", web::put().to(update))
        .route("/test-telegram", web::post().to(test_telegram))
        .route("/test-email", web::post().to(test_email))
}

async fn get_all(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let settings = settings_service::get_all_settings(&db).await?;
    let map: HashMap<String, String> = settings.into_iter().map(|s| (s.key, s.value)).collect();
    Ok(HttpResponse::Ok().json(map))
}

async fn update(
    db: web::Data<DatabaseConnection>,
    body: web::Json<HashMap<String, String>>,
) -> AppResult<HttpResponse> {
    let settings = body.into_inner();
    for (key, value) in &settings {
        settings_service::set_setting(&db, key, value).await?;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

async fn test_telegram(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let bot_token = settings_service::get_setting(db.get_ref(), "telegram_bot_token")
        .await?
        .unwrap_or_default();
    let chat_id = settings_service::get_setting(db.get_ref(), "telegram_chat_id")
        .await?
        .unwrap_or_default();

    let config = aff_notify::telegram::TelegramConfig {
        bot_token,
        chat_id,
        enabled: true,
    };

    match aff_notify::telegram::send_message(&config, "AFF Card Shop - Test notification. Telegram is configured correctly.").await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": true}))),
        Err(e) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": false, "error": e}))),
    }
}

async fn test_email(
    db: web::Data<DatabaseConnection>,
    body: web::Json<TestNotifyDto>,
) -> AppResult<HttpResponse> {
    let host = settings_service::get_setting(db.get_ref(), "smtp_host")
        .await?
        .unwrap_or_default();
    let port: u16 = settings_service::get_setting(db.get_ref(), "smtp_port")
        .await?
        .unwrap_or_else(|| "465".to_string())
        .parse()
        .unwrap_or(465);
    let username = settings_service::get_setting(db.get_ref(), "smtp_username")
        .await?
        .unwrap_or_default();
    let password = settings_service::get_setting(db.get_ref(), "smtp_password")
        .await?
        .unwrap_or_default();
    let from_address = settings_service::get_setting(db.get_ref(), "smtp_from")
        .await?
        .unwrap_or_default();

    let config = aff_notify::email::SmtpConfig {
        host,
        port,
        username,
        password,
        from_address,
        enabled: true,
    };

    let to = body.target.clone().unwrap_or_else(|| config.from_address.clone());

    match aff_notify::email::send_test_email(&config, &to).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": true}))),
        Err(e) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": false, "error": e}))),
    }
}
