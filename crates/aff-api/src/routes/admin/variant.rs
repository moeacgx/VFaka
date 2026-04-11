use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;

use aff_common::error::AppResult;
use aff_core::services::variant_service;
use aff_entity::dto::variant_dto::{CreateVariantDto, UpdateVariantDto};

pub fn scope() -> actix_web::Scope {
    web::scope("/variants")
        .route("/product/{product_id}", web::get().to(list_by_product))
        .route("/product/{product_id}", web::post().to(create))
        .route("/{id}", web::get().to(get))
        .route("/{id}", web::put().to(update))
        .route("/{id}", web::delete().to(delete))
}

async fn list_by_product(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let variants = variant_service::list_variants(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(variants))
}

async fn get(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let variant = variant_service::get_variant(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(variant))
}

async fn create(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<CreateVariantDto>,
) -> AppResult<HttpResponse> {
    let variant =
        variant_service::create_variant(&db, path.into_inner(), body.into_inner()).await?;
    Ok(HttpResponse::Created().json(variant))
}

async fn update(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<UpdateVariantDto>,
) -> AppResult<HttpResponse> {
    let variant =
        variant_service::update_variant(&db, path.into_inner(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(variant))
}

async fn delete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    variant_service::delete_variant(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
