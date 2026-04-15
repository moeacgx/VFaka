pub mod m20260406_000001_init;
pub mod m20260406_000002_aff_tiers;
pub mod m20260406_000003_media_and_notify;
pub mod m20260406_000004_coupons;
pub mod m20260406_000005_security_hardening;
pub mod m20260411_000006_product_variants;
pub mod m20260416_000007_product_delivery_mode;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260406_000001_init::Migration),
            Box::new(m20260406_000002_aff_tiers::Migration),
            Box::new(m20260406_000003_media_and_notify::Migration),
            Box::new(m20260406_000004_coupons::Migration),
            Box::new(m20260406_000005_security_hardening::Migration),
            Box::new(m20260411_000006_product_variants::Migration),
            Box::new(m20260416_000007_product_delivery_mode::Migration),
        ]
    }
}
