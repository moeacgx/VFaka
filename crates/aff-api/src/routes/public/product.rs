use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::{category_service, product_service};

#[derive(Debug, Deserialize)]
pub struct ProductListQuery {
    pub category_id: Option<i32>,
}

pub async fn list_categories(
    db: web::Data<DatabaseConnection>,
) -> AppResult<HttpResponse> {
    let cats = category_service::list_categories(db.get_ref()).await?;
    // Filter to active only
    let active: Vec<_> = cats.into_iter().filter(|c| c.is_active).collect();
    Ok(HttpResponse::Ok().json(active))
}

pub async fn list_products(
    db: web::Data<DatabaseConnection>,
    query: web::Query<ProductListQuery>,
) -> AppResult<HttpResponse> {
    let products = product_service::list_products(db.get_ref(), query.category_id).await?;
    // Filter to active only for public API
    let active: Vec<_> = products.into_iter().filter(|p| p.is_active).collect();
    Ok(HttpResponse::Ok().json(active))
}

pub async fn get_product(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let product = product_service::get_product(db.get_ref(), id).await?;
    if !product.is_active {
        return Err(aff_common::error::AppError::NotFound(format!(
            "Product {} not found",
            id
        )));
    }
    Ok(HttpResponse::Ok().json(product))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/categories", web::get().to(list_categories))
        .route("/products", web::get().to(list_products))
        .route("/products/{id}", web::get().to(get_product));
}
