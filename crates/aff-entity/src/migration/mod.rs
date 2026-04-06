pub mod m20260406_000001_init;
pub mod m20260406_000002_aff_tiers;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260406_000001_init::Migration),
            Box::new(m20260406_000002_aff_tiers::Migration),
        ]
    }
}
