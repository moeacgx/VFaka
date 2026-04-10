use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create coupons table
        manager
            .create_table(
                Table::create()
                    .table(Coupons::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Coupons::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Coupons::Code)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Coupons::DiscountType)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Coupons::DiscountValue).double().not_null())
                    .col(ColumnDef::new(Coupons::ProductId).integer().null())
                    .col(
                        ColumnDef::new(Coupons::MinAmount)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .col(ColumnDef::new(Coupons::MaxUses).integer().null())
                    .col(
                        ColumnDef::new(Coupons::UsedCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Coupons::ValidFrom).date_time().null())
                    .col(ColumnDef::new(Coupons::ValidTo).date_time().null())
                    .col(
                        ColumnDef::new(Coupons::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Coupons::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add coupon_code and discount_amount to orders
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column(
                        ColumnDef::new(Orders::CouponCode)
                            .string_len(64)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column(
                        ColumnDef::new(Orders::DiscountAmount)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Coupons::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Coupons {
    Table,
    Id,
    Code,
    DiscountType,
    DiscountValue,
    ProductId,
    MinAmount,
    MaxUses,
    UsedCount,
    ValidFrom,
    ValidTo,
    IsActive,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    CouponCode,
    DiscountAmount,
}
