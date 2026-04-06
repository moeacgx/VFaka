use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::order_service;

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub status: Option<String>,
    pub email: Option<String>,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/orders")
        .route("", web::get().to(list))
        .route("/{id}", web::get().to(get))
}

async fn list(
    db: web::Data<DatabaseConnection>,
    query: web::Query<OrderListQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let (orders, total) =
        order_service::list_orders(&db, page, per_page, query.status.clone(), query.email.clone())
            .await?;

    let items = order_service::enrich_orders_with_product_names(&db, orders).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": items,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

async fn get(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let order = order_service::get_order(&db, path.into_inner()).await?;
    let items = order_service::enrich_orders_with_product_names(&db, vec![order]).await?;
    Ok(HttpResponse::Ok().json(items.into_iter().next()))
}
