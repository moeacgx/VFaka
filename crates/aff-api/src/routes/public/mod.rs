pub mod aff;
pub mod announcement;
pub mod callback;
pub mod coupon;
pub mod order;
pub mod product;

use actix_web::web;
use actix_governor::{Governor, GovernorConfigBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Rate limit: 10 requests per 60 seconds per IP for sensitive endpoints
    let rate_limit_conf = GovernorConfigBuilder::default()
        .seconds_per_request(6)
        .burst_size(10)
        .finish()
        .unwrap();

    cfg.service(
        web::scope("/v1")
            .configure(product::configure)
            .configure(callback::configure)
            .configure(announcement::configure)
            // Rate-limited routes (orders, aff, coupons)
            .service(
                web::scope("")
                    .wrap(Governor::new(&rate_limit_conf))
                    .configure(order::configure)
                    .configure(aff::configure)
                    .configure(coupon::configure),
            ),
    );
}
