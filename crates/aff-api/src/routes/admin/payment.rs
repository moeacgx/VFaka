use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::payment_config_service;

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentConfigDto {
    pub config_json: String,
    pub is_active: bool,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payment-configs")
            .route("", web::get().to(list))
            .route("/{channel}", web::put().to(update)),
    );
}

async fn list(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let configs = payment_config_service::list_configs(&db).await?;
    Ok(HttpResponse::Ok().json(configs))
}

async fn update(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    body: web::Json<UpdatePaymentConfigDto>,
) -> AppResult<HttpResponse> {
    let channel = path.into_inner();
    let dto = body.into_inner();
    let config =
        payment_config_service::update_config(&db, &channel, &dto.config_json, dto.is_active)
            .await?;
    Ok(HttpResponse::Ok().json(config))
}
