use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::config::AppConfig;
use aff_common::error::AppResult;
use aff_core::services::admin_service;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    db: web::Data<DatabaseConnection>,
    config: web::Data<AppConfig>,
    body: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    let (admin, token) =
        admin_service::login(db.get_ref(), config.get_ref(), &body.username, &body.password)
            .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "admin": {
            "id": admin.id,
            "username": admin.username,
            "role": admin.role,
        },
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login)),
    );
}
