use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create aff_tiers table
        manager
            .create_table(
                Table::create()
                    .table(AffTiers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AffTiers::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(AffTiers::Level).integer().not_null().unique_key())
                    .col(ColumnDef::new(AffTiers::Name).string_len(64).not_null())
                    .col(ColumnDef::new(AffTiers::CommissionRate).double().not_null())
                    .col(ColumnDef::new(AffTiers::RequiredAmount).double().not_null().default(0.0))
                    .to_owned(),
            )
            .await?;

        // Add level column to aff_users (default 1)
        manager
            .alter_table(
                Table::alter()
                    .table(AffUsers::Table)
                    .add_column(ColumnDef::new(AffUsers::Level).integer().not_null().default(1))
                    .to_owned(),
            )
            .await?;

        // Seed default tiers
        let insert = Query::insert()
            .into_table(AffTiers::Table)
            .columns([
                AffTiers::Level,
                AffTiers::Name,
                AffTiers::CommissionRate,
                AffTiers::RequiredAmount,
            ])
            .values_panic(vec![1.into(), "青铜".into(), 0.05.into(), 0.0.into()])
            .values_panic(vec![2.into(), "白银".into(), 0.10.into(), 100.0.into()])
            .values_panic(vec![3.into(), "黄金".into(), 0.15.into(), 500.0.into()])
            .values_panic(vec![4.into(), "钻石".into(), 0.20.into(), 1000.0.into()])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AffTiers::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum AffTiers {
    Table,
    Id,
    Level,
    Name,
    CommissionRate,
    RequiredAmount,
}

#[derive(Iden)]
enum AffUsers {
    Table,
    Level,
}
