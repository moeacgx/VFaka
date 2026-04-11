pub mod admin_mgmt;
pub mod aff;
pub mod auth;
pub mod card;
pub mod category;
pub mod coupon;
pub mod dashboard;
pub mod order;
pub mod payment;
pub mod product;
pub mod settings;
pub mod upload;
pub mod variant;
pub mod withdrawal;

use actix_web::web;
use crate::middleware::auth::{JwtAuth, SuperAdminAuth};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // Auth routes (no JWT required)
            .configure(auth::configure)
            // Super admin routes
            .service(settings::scope().wrap(SuperAdminAuth))
            .service(payment::scope().wrap(SuperAdminAuth))
            .service(admin_mgmt::scope().wrap(SuperAdminAuth))
            // Regular admin routes
            .service(dashboard::scope().wrap(JwtAuth))
            .service(category::scope().wrap(JwtAuth))
            .service(product::scope().wrap(JwtAuth))
            .service(variant::scope().wrap(JwtAuth))
            .service(card::scope().wrap(JwtAuth))
            .service(order::scope().wrap(JwtAuth))
            .service(coupon::scope().wrap(JwtAuth))
            .service(withdrawal::scope().wrap(JwtAuth))
            .service(aff::scope().wrap(JwtAuth))
            .service(upload::scope().wrap(JwtAuth)),
    );
}
