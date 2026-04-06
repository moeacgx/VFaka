use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;

use aff_common::error::AppResult;
use aff_core::services::{aff_service, settings_service};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/aff")
            .route("/users", web::get().to(list_users))
            .route("/settings", web::put().to(update_settings)),
    );
}

async fn list_users(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let users = aff_service::list_aff_users(&db).await?;
    Ok(HttpResponse::Ok().json(users))
}

async fn update_settings(
    db: web::Data<DatabaseConnection>,
    body: web::Json<HashMap<String, String>>,
) -> AppResult<HttpResponse> {
    let settings = body.into_inner();
    for (key, value) in &settings {
        settings_service::set_setting(&db, key, value).await?;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
