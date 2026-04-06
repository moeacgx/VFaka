use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::{settings_service, withdrawal_service};

#[derive(Debug, Deserialize)]
pub struct WithdrawalListQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectBody {
    pub note: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteBody {
    pub tx_hash: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/withdrawals")
            .route("", web::get().to(list))
            .route("/{id}/approve", web::put().to(approve))
            .route("/{id}/reject", web::put().to(reject))
            .route("/{id}/complete", web::put().to(complete)),
    );
}

async fn list(
    db: web::Data<DatabaseConnection>,
    query: web::Query<WithdrawalListQuery>,
) -> AppResult<HttpResponse> {
    let items = withdrawal_service::list_withdrawals(&db, query.status.clone()).await?;
    Ok(HttpResponse::Ok().json(items))
}

async fn approve(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let item = withdrawal_service::approve_withdrawal(&db, path.into_inner()).await?;
    send_withdrawal_email(&db, &item, None).await;
    Ok(HttpResponse::Ok().json(item))
}

async fn reject(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<RejectBody>,
) -> AppResult<HttpResponse> {
    let item =
        withdrawal_service::reject_withdrawal(&db, path.into_inner(), &body.note).await?;
    send_withdrawal_email(&db, &item, None).await;
    Ok(HttpResponse::Ok().json(item))
}

async fn complete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<CompleteBody>,
) -> AppResult<HttpResponse> {
    let item =
        withdrawal_service::complete_withdrawal(&db, path.into_inner(), &body.tx_hash).await?;
    send_withdrawal_email(&db, &item, Some(body.tx_hash.clone())).await;
    Ok(HttpResponse::Ok().json(item))
}

async fn send_withdrawal_email(
    db: &DatabaseConnection,
    withdrawal: &aff_entity::entities::withdrawal::Model,
    tx_hash: Option<String>,
) {
    let smtp_enabled = settings_service::get_setting(db, "smtp_enabled")
        .await.ok().flatten().unwrap_or_default() == "true";
    if !smtp_enabled {
        return;
    }

    let host = settings_service::get_setting(db, "smtp_host").await.ok().flatten().unwrap_or_default();
    let port: u16 = settings_service::get_setting(db, "smtp_port").await.ok().flatten()
        .unwrap_or_else(|| "465".to_string()).parse().unwrap_or(465);
    let username = settings_service::get_setting(db, "smtp_username").await.ok().flatten().unwrap_or_default();
    let password = settings_service::get_setting(db, "smtp_password").await.ok().flatten().unwrap_or_default();
    let from_address = settings_service::get_setting(db, "smtp_from").await.ok().flatten().unwrap_or_default();

    // Look up aff user email
    use sea_orm::EntityTrait;
    let aff_user = aff_entity::entities::aff_user::Entity::find_by_id(withdrawal.aff_user_id)
        .one(db)
        .await
        .ok()
        .flatten();

    if let Some(user) = aff_user {
        let config = aff_notify::email::SmtpConfig {
            host, port, username, password, from_address, enabled: true,
        };
        aff_notify::email::send_withdrawal_status(
            config,
            user.email,
            withdrawal.amount,
            withdrawal.currency.clone(),
            withdrawal.status.clone(),
            tx_hash,
            withdrawal.admin_note.clone(),
        );
    }
}
