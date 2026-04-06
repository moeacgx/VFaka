use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;

use aff_common::error::AppResult;
use aff_core::services::settings_service;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .route("", web::get().to(get_all))
            .route("", web::put().to(update)),
    );
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
