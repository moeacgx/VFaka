pub mod public;
pub mod admin;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(public::configure)
            .configure(admin::configure),
    );
}
