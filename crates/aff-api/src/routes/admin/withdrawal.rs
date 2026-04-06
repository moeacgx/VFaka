use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::withdrawal_service;

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
    Ok(HttpResponse::Ok().json(item))
}

async fn reject(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<RejectBody>,
) -> AppResult<HttpResponse> {
    let item =
        withdrawal_service::reject_withdrawal(&db, path.into_inner(), &body.note).await?;
    Ok(HttpResponse::Ok().json(item))
}

async fn complete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<CompleteBody>,
) -> AppResult<HttpResponse> {
    let item =
        withdrawal_service::complete_withdrawal(&db, path.into_inner(), &body.tx_hash).await?;
    Ok(HttpResponse::Ok().json(item))
}
