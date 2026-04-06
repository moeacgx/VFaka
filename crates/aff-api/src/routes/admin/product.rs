use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::{card_service, product_service};
use aff_entity::dto::{CreateProductDto, ImportCardsDto, UpdateProductDto};

#[derive(Debug, Deserialize)]
pub struct ProductListQuery {
    pub category_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteDto {
    pub ids: Vec<i32>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .route("", web::get().to(list))
            .route("", web::post().to(create))
            .route("/batch-delete", web::post().to(batch_delete))
            .route("/{id}", web::get().to(get))
            .route("/{id}", web::put().to(update))
            .route("/{id}", web::delete().to(delete))
            .route("/{id}/restock", web::post().to(restock)),
    );
}

async fn list(
    db: web::Data<DatabaseConnection>,
    query: web::Query<ProductListQuery>,
) -> AppResult<HttpResponse> {
    let products = product_service::list_products(&db, query.category_id).await?;
    Ok(HttpResponse::Ok().json(products))
}

async fn create(
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateProductDto>,
) -> AppResult<HttpResponse> {
    let product = product_service::create_product(&db, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(product))
}

async fn get(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let product = product_service::get_product(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(product))
}

async fn update(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<UpdateProductDto>,
) -> AppResult<HttpResponse> {
    let product =
        product_service::update_product(&db, path.into_inner(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(product))
}

async fn delete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    product_service::delete_product(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"ok": true})))
}

async fn batch_delete(
    db: web::Data<DatabaseConnection>,
    body: web::Json<BatchDeleteDto>,
) -> AppResult<HttpResponse> {
    let count = product_service::batch_delete_products(&db, body.into_inner().ids).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"ok": true, "deleted": count})))
}

async fn restock(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<ImportCardsDto>,
) -> AppResult<HttpResponse> {
    let product_id = path.into_inner();
    let dto = body.into_inner();
    let count = card_service::import_cards(&db, product_id, &dto.cards).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "imported": count,
    })))
}
