use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::coupon_service;
use aff_entity::dto::{CreateCouponDto, UpdateCouponDto};

#[derive(Debug, Deserialize)]
pub struct CouponListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub product_id: Option<i32>,
    pub is_active: Option<bool>,
}

async fn list_coupons(
    db: web::Data<DatabaseConnection>,
    query: web::Query<CouponListQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let (items, total) = coupon_service::list_coupons(
        db.get_ref(),
        page,
        per_page,
        query.product_id,
        query.is_active,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": items,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

async fn create_coupon(
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateCouponDto>,
) -> AppResult<HttpResponse> {
    let coupon = coupon_service::create_coupon(db.get_ref(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(coupon))
}

async fn update_coupon(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<UpdateCouponDto>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let coupon = coupon_service::update_coupon(db.get_ref(), id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(coupon))
}

async fn delete_coupon(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    coupon_service::delete_coupon(db.get_ref(), id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteDto {
    pub ids: Vec<i32>,
}

async fn batch_delete_coupons(
    db: web::Data<DatabaseConnection>,
    body: web::Json<BatchDeleteDto>,
) -> AppResult<HttpResponse> {
    let count = coupon_service::batch_delete_coupons(db.get_ref(), body.into_inner().ids).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "deleted": count,
    })))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/coupons")
        .route("", web::get().to(list_coupons))
        .route("", web::post().to(create_coupon))
        .route("/{id}", web::put().to(update_coupon))
        .route("/{id}", web::delete().to(delete_coupon))
        .route("/batch-delete", web::post().to(batch_delete_coupons))
}
