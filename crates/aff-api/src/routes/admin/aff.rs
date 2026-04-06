use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;

use aff_common::error::AppResult;
use aff_core::services::{aff_service, settings_service};
use aff_entity::dto::{CreateAffTierDto, UpdateAffTierDto};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/aff")
            .route("/users", web::get().to(list_users))
            .route("/settings", web::put().to(update_settings))
            .route("/tiers", web::get().to(list_tiers))
            .route("/tiers", web::post().to(create_tier))
            .route("/tiers/{level}", web::put().to(update_tier))
            .route("/tiers/{level}", web::delete().to(delete_tier)),
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

async fn list_tiers(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let tiers = aff_service::list_tiers(&db).await?;
    Ok(HttpResponse::Ok().json(tiers))
}

async fn create_tier(
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateAffTierDto>,
) -> AppResult<HttpResponse> {
    let tier = aff_service::create_tier(&db, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(tier))
}

async fn update_tier(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<UpdateAffTierDto>,
) -> AppResult<HttpResponse> {
    let level = path.into_inner();
    let tier = aff_service::update_tier(&db, level, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(tier))
}

async fn delete_tier(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let level = path.into_inner();
    aff_service::delete_tier(&db, level).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
