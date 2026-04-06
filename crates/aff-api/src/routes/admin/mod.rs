pub mod admin_mgmt;
pub mod aff;
pub mod auth;
pub mod card;
pub mod category;
pub mod dashboard;
pub mod order;
pub mod payment;
pub mod product;
pub mod settings;
pub mod upload;
pub mod withdrawal;

use actix_web::web;
use crate::middleware::auth::{JwtAuth, SuperAdminAuth};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // Auth routes (no JWT required)
            .configure(auth::configure)
            // Super admin routes (admin management, settings, payment config)
            .service(
                web::scope("")
                    .wrap(SuperAdminAuth)
                    .configure(settings::configure)
                    .configure(payment::configure)
                    .configure(admin_mgmt::configure),
            )
            // Regular admin routes (require JWT but any role)
            .service(
                web::scope("")
                    .wrap(JwtAuth)
                    .configure(dashboard::configure)
                    .configure(category::configure)
                    .configure(product::configure)
                    .configure(card::configure)
                    .configure(order::configure)
                    .configure(withdrawal::configure)
                    .configure(aff::configure)
                    .configure(upload::configure),
            ),
    );
}
