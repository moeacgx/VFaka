use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;

use aff_common::error::AppResult;
use aff_core::services::category_service;
use aff_entity::dto::{CreateCategoryDto, UpdateCategoryDto};

#[derive(serde::Deserialize)]
pub struct BatchDeleteDto {
    pub ids: Vec<i32>,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/categories")
        .route("", web::get().to(list))
        .route("", web::post().to(create))
        .route("/batch-delete", web::post().to(batch_delete))
        .route("/{id}", web::get().to(get))
        .route("/{id}", web::put().to(update))
        .route("/{id}", web::delete().to(delete))
}

async fn list(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let cats = category_service::list_categories(&db).await?;
    Ok(HttpResponse::Ok().json(cats))
}

async fn create(
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateCategoryDto>,
) -> AppResult<HttpResponse> {
    let cat = category_service::create_category(&db, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(cat))
}

async fn get(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let cat = category_service::get_category(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(cat))
}

async fn update(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<UpdateCategoryDto>,
) -> AppResult<HttpResponse> {
    let cat = category_service::update_category(&db, path.into_inner(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(cat))
}

async fn delete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    category_service::delete_category(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"ok": true})))
}

async fn batch_delete(
    db: web::Data<DatabaseConnection>,
    body: web::Json<BatchDeleteDto>,
) -> AppResult<HttpResponse> {
    let count = category_service::batch_delete_categories(&db, body.into_inner().ids).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"ok": true, "deleted": count})))
}
