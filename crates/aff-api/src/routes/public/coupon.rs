use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;

use aff_common::error::AppResult;
use aff_core::services::coupon_service;
use aff_entity::dto::ValidateCouponDto;

async fn validate_coupon(
    db: web::Data<DatabaseConnection>,
    body: web::Json<ValidateCouponDto>,
) -> AppResult<HttpResponse> {
    let dto = body.into_inner();
    let result =
        coupon_service::validate_coupon(db.get_ref(), &dto.code, dto.product_id, dto.amount)
            .await?;
    Ok(HttpResponse::Ok().json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/coupons/validate", web::post().to(validate_coupon));
}
