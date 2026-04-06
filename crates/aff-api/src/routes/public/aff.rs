use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;

use aff_common::error::{AppError, AppResult};
use aff_core::services::{aff_service, settings_service, withdraw_service};
use aff_entity::dto::{AffRegisterDto, AffWithdrawDto};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AffEmailQuery {
    pub email: String,
}

pub async fn register(
    db: web::Data<DatabaseConnection>,
    body: web::Json<AffRegisterDto>,
) -> AppResult<HttpResponse> {
    let dto = body.into_inner();

    if dto.email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }
    if dto.withdraw_password.is_empty() {
        return Err(AppError::BadRequest("Withdraw password is required".into()));
    }

    let user = aff_service::register(db.get_ref(), dto).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "aff_code": user.aff_code,
        "email": user.email,
    })))
}

pub async fn query(
    db: web::Data<DatabaseConnection>,
    query: web::Query<AffEmailQuery>,
) -> AppResult<HttpResponse> {
    if query.email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }

    let resp = aff_service::query_by_email(db.get_ref(), &query.email).await?;

    Ok(HttpResponse::Ok().json(resp))
}

pub async fn withdraw(
    db: web::Data<DatabaseConnection>,
    body: web::Json<AffWithdrawDto>,
) -> AppResult<HttpResponse> {
    let dto = body.into_inner();

    if dto.email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }
    if dto.amount <= 0.0 {
        return Err(AppError::BadRequest("Amount must be positive".into()));
    }

    let withdrawal = withdraw_service::create_withdrawal(db.get_ref(), dto).await?;

    // Send Telegram notification for new withdrawal request
    let tg_enabled = settings_service::get_setting(db.get_ref(), "telegram_enabled")
        .await.ok().flatten().unwrap_or_default() == "true";
    if tg_enabled {
        let bot_token = settings_service::get_setting(db.get_ref(), "telegram_bot_token")
            .await.ok().flatten().unwrap_or_default();
        let chat_id = settings_service::get_setting(db.get_ref(), "telegram_chat_id")
            .await.ok().flatten().unwrap_or_default();
        let config = aff_notify::telegram::TelegramConfig { bot_token, chat_id, enabled: true };
        aff_notify::telegram::send_withdrawal_notification(
            config,
            withdrawal.wallet_address.clone(),
            withdrawal.amount,
            withdrawal.currency.clone(),
            withdrawal.chain.clone(),
            withdrawal.wallet_address.clone(),
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": withdrawal.id,
        "amount": withdrawal.amount,
        "currency": withdrawal.currency,
        "chain": withdrawal.chain,
        "status": withdrawal.status,
    })))
}

pub async fn logs(
    db: web::Data<DatabaseConnection>,
    query: web::Query<AffEmailQuery>,
) -> AppResult<HttpResponse> {
    if query.email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }

    let logs = aff_service::get_logs(db.get_ref(), &query.email).await?;

    Ok(HttpResponse::Ok().json(logs))
}

pub async fn tiers(
    db: web::Data<DatabaseConnection>,
) -> AppResult<HttpResponse> {
    let tiers = aff_service::list_tiers(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(tiers))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/aff")
            .route("/register", web::post().to(register))
            .route("/query", web::get().to(query))
            .route("/withdraw", web::post().to(withdraw))
            .route("/logs", web::get().to(logs))
            .route("/tiers", web::get().to(tiers)),
    );
}
