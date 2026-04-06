pub mod aff;
pub mod announcement;
pub mod callback;
pub mod order;
pub mod product;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1")
            .configure(product::configure)
            .configure(order::configure)
            .configure(callback::configure)
            .configure(aff::configure)
            .configure(announcement::configure),
    );
}
