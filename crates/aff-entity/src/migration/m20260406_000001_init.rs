use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. admins
        manager.create_table(
            Table::create()
                .table(Admins::Table)
                .if_not_exists()
                .col(ColumnDef::new(Admins::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Admins::Username).string_len(64).not_null().unique_key())
                .col(ColumnDef::new(Admins::PasswordHash).string_len(255).not_null())
                .col(ColumnDef::new(Admins::Role).string_len(32).not_null().default("admin"))
                .col(ColumnDef::new(Admins::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Admins::UpdatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        // 2. categories
        manager.create_table(
            Table::create()
                .table(Categories::Table)
                .if_not_exists()
                .col(ColumnDef::new(Categories::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Categories::Name).string_len(128).not_null())
                .col(ColumnDef::new(Categories::SortOrder).integer().not_null().default(0))
                .col(ColumnDef::new(Categories::IsActive).boolean().not_null().default(true))
                .col(ColumnDef::new(Categories::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        // 3. products
        manager.create_table(
            Table::create()
                .table(Products::Table)
                .if_not_exists()
                .col(ColumnDef::new(Products::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Products::CategoryId).integer().null())
                .col(ColumnDef::new(Products::Name).string_len(255).not_null())
                .col(ColumnDef::new(Products::Description).text().null())
                .col(ColumnDef::new(Products::Price).double().not_null())
                .col(ColumnDef::new(Products::StockCount).integer().not_null().default(0))
                .col(ColumnDef::new(Products::SalesCount).integer().not_null().default(0))
                .col(ColumnDef::new(Products::IsActive).boolean().not_null().default(true))
                .col(ColumnDef::new(Products::AllowAlipay).boolean().not_null().default(true))
                .col(ColumnDef::new(Products::AllowWxpay).boolean().not_null().default(true))
                .col(ColumnDef::new(Products::AllowQqpay).boolean().not_null().default(true))
                .col(ColumnDef::new(Products::AllowUsdtTrc20).boolean().not_null().default(true))
                .col(ColumnDef::new(Products::AllowTrx).boolean().not_null().default(true))
                .col(ColumnDef::new(Products::AllowUsdtErc20).boolean().not_null().default(true))
                .col(ColumnDef::new(Products::PostPayActionType).string_len(32).null())
                .col(ColumnDef::new(Products::PostPayActionValue).text().null())
                .col(ColumnDef::new(Products::AffCommissionRate).double().null())
                .col(ColumnDef::new(Products::SortOrder).integer().not_null().default(0))
                .col(ColumnDef::new(Products::MinQuantity).integer().not_null().default(1))
                .col(ColumnDef::new(Products::MaxQuantity).integer().not_null().default(10))
                .col(ColumnDef::new(Products::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Products::UpdatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Products::Table, Products::CategoryId)
                        .to(Categories::Table, Categories::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        ).await?;

        // 4. orders (created before cards so cards can reference orders)
        manager.create_table(
            Table::create()
                .table(Orders::Table)
                .if_not_exists()
                .col(ColumnDef::new(Orders::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Orders::OrderNo).string_len(64).not_null().unique_key())
                .col(ColumnDef::new(Orders::ProductId).integer().not_null())
                .col(ColumnDef::new(Orders::Quantity).integer().not_null().default(1))
                .col(ColumnDef::new(Orders::TotalAmount).double().not_null())
                .col(ColumnDef::new(Orders::Email).string_len(255).not_null())
                .col(ColumnDef::new(Orders::PaymentMethod).string_len(32).not_null())
                .col(ColumnDef::new(Orders::PaymentChannel).string_len(32).not_null())
                .col(ColumnDef::new(Orders::Status).string_len(32).not_null().default("pending"))
                .col(ColumnDef::new(Orders::TradeNo).string_len(128).null())
                .col(ColumnDef::new(Orders::PayTime).date_time().null())
                .col(ColumnDef::new(Orders::AffCode).string_len(32).null())
                .col(ColumnDef::new(Orders::AffUserEmail).string_len(255).null())
                .col(ColumnDef::new(Orders::AffCommission).double().not_null().default(0.0))
                .col(ColumnDef::new(Orders::CardsSnapshot).text().null())
                .col(ColumnDef::new(Orders::PostActionResult).text().null())
                .col(ColumnDef::new(Orders::IpAddress).string_len(64).null())
                .col(ColumnDef::new(Orders::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Orders::UpdatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Orders::Table, Orders::ProductId)
                        .to(Products::Table, Products::Id),
                )
                .to_owned(),
        ).await?;

        // 5. cards
        manager.create_table(
            Table::create()
                .table(Cards::Table)
                .if_not_exists()
                .col(ColumnDef::new(Cards::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Cards::ProductId).integer().not_null())
                .col(ColumnDef::new(Cards::Content).text().not_null())
                .col(ColumnDef::new(Cards::Status).string_len(32).not_null().default("available"))
                .col(ColumnDef::new(Cards::OrderId).integer().null())
                .col(ColumnDef::new(Cards::SoldAt).date_time().null())
                .col(ColumnDef::new(Cards::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Cards::Table, Cards::ProductId)
                        .to(Products::Table, Products::Id),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Cards::Table, Cards::OrderId)
                        .to(Orders::Table, Orders::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        ).await?;

        // 6. aff_users
        manager.create_table(
            Table::create()
                .table(AffUsers::Table)
                .if_not_exists()
                .col(ColumnDef::new(AffUsers::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(AffUsers::Email).string_len(255).not_null().unique_key())
                .col(ColumnDef::new(AffUsers::AffCode).string_len(32).not_null().unique_key())
                .col(ColumnDef::new(AffUsers::Balance).double().not_null().default(0.0))
                .col(ColumnDef::new(AffUsers::TotalEarned).double().not_null().default(0.0))
                .col(ColumnDef::new(AffUsers::TotalWithdrawn).double().not_null().default(0.0))
                .col(ColumnDef::new(AffUsers::WithdrawPasswordHash).string_len(255).null())
                .col(ColumnDef::new(AffUsers::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        // 7. aff_logs
        manager.create_table(
            Table::create()
                .table(AffLogs::Table)
                .if_not_exists()
                .col(ColumnDef::new(AffLogs::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(AffLogs::AffUserId).integer().not_null())
                .col(ColumnDef::new(AffLogs::OrderId).integer().not_null())
                .col(ColumnDef::new(AffLogs::Commission).double().not_null())
                .col(ColumnDef::new(AffLogs::Status).string_len(32).not_null().default("credited"))
                .col(ColumnDef::new(AffLogs::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(AffLogs::Table, AffLogs::AffUserId)
                        .to(AffUsers::Table, AffUsers::Id),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(AffLogs::Table, AffLogs::OrderId)
                        .to(Orders::Table, Orders::Id),
                )
                .to_owned(),
        ).await?;

        // 8. withdrawals
        manager.create_table(
            Table::create()
                .table(Withdrawals::Table)
                .if_not_exists()
                .col(ColumnDef::new(Withdrawals::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Withdrawals::AffUserId).integer().not_null())
                .col(ColumnDef::new(Withdrawals::Amount).double().not_null())
                .col(ColumnDef::new(Withdrawals::Currency).string_len(16).not_null())
                .col(ColumnDef::new(Withdrawals::Chain).string_len(32).not_null())
                .col(ColumnDef::new(Withdrawals::WalletAddress).string_len(128).not_null())
                .col(ColumnDef::new(Withdrawals::Status).string_len(32).not_null().default("pending"))
                .col(ColumnDef::new(Withdrawals::AdminNote).text().null())
                .col(ColumnDef::new(Withdrawals::TxHash).string_len(128).null())
                .col(ColumnDef::new(Withdrawals::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Withdrawals::ProcessedAt).date_time().null())
                .foreign_key(
                    ForeignKey::create()
                        .from(Withdrawals::Table, Withdrawals::AffUserId)
                        .to(AffUsers::Table, AffUsers::Id),
                )
                .to_owned(),
        ).await?;

        // 9. payment_configs
        manager.create_table(
            Table::create()
                .table(PaymentConfigs::Table)
                .if_not_exists()
                .col(ColumnDef::new(PaymentConfigs::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(PaymentConfigs::Channel).string_len(32).not_null().unique_key())
                .col(ColumnDef::new(PaymentConfigs::IsActive).boolean().not_null().default(true))
                .col(ColumnDef::new(PaymentConfigs::ConfigJson).text().not_null())
                .col(ColumnDef::new(PaymentConfigs::CreatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(PaymentConfigs::UpdatedAt).date_time().not_null().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        // 10. system_settings
        manager.create_table(
            Table::create()
                .table(SystemSettings::Table)
                .if_not_exists()
                .col(ColumnDef::new(SystemSettings::Key).string_len(128).not_null().primary_key())
                .col(ColumnDef::new(SystemSettings::Value).text().not_null())
                .to_owned(),
        ).await?;

        // Insert default system settings
        let insert = Query::insert()
            .into_table(SystemSettings::Table)
            .columns([SystemSettings::Key, SystemSettings::Value])
            .values_panic(["site_name".into(), "AFF Card Shop".into()])
            .values_panic(["site_description".into(), "自动发卡商城".into()])
            .values_panic(["global_aff_commission_rate".into(), "0.05".into()])
            .values_panic(["min_withdraw_amount".into(), "10".into()])
            .values_panic(["withdraw_fee_rate".into(), "0".into()])
            .to_owned();
        manager.exec_stmt(insert).await?;

        // Insert default payment configs
        let insert_pay = Query::insert()
            .into_table(PaymentConfigs::Table)
            .columns([PaymentConfigs::Channel, PaymentConfigs::IsActive, PaymentConfigs::ConfigJson])
            .values_panic(["epay".into(), true.into(), r#"{"pid":"","key":"","api_url":"https://pay.myzfw.com"}"#.into()])
            .values_panic(["tokenpay".into(), true.into(), r#"{"api_url":"http://tokenpay:5000","notify_secret":"","custom_domain":""}"#.into()])
            .to_owned();
        manager.exec_stmt(insert_pay).await?;

        // Create indexes for common queries
        manager.create_index(
            Index::create().name("idx_orders_email").table(Orders::Table).col(Orders::Email).to_owned(),
        ).await?;
        manager.create_index(
            Index::create().name("idx_orders_status").table(Orders::Table).col(Orders::Status).to_owned(),
        ).await?;
        manager.create_index(
            Index::create().name("idx_cards_product_status").table(Cards::Table).col(Cards::ProductId).col(Cards::Status).to_owned(),
        ).await?;
        manager.create_index(
            Index::create().name("idx_aff_logs_user").table(AffLogs::Table).col(AffLogs::AffUserId).to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(AffLogs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Withdrawals::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(AffUsers::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Cards::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Orders::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Products::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Categories::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Admins::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(PaymentConfigs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SystemSettings::Table).to_owned()).await?;
        Ok(())
    }
}

// Iden enums for type-safe table/column references

#[derive(DeriveIden)]
enum Admins {
    Table,
    Id,
    Username,
    PasswordHash,
    Role,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    Name,
    SortOrder,
    IsActive,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    CategoryId,
    Name,
    Description,
    Price,
    StockCount,
    SalesCount,
    IsActive,
    AllowAlipay,
    AllowWxpay,
    AllowQqpay,
    AllowUsdtTrc20,
    AllowTrx,
    AllowUsdtErc20,
    PostPayActionType,
    PostPayActionValue,
    AffCommissionRate,
    SortOrder,
    MinQuantity,
    MaxQuantity,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    OrderNo,
    ProductId,
    Quantity,
    TotalAmount,
    Email,
    PaymentMethod,
    PaymentChannel,
    Status,
    TradeNo,
    PayTime,
    AffCode,
    AffUserEmail,
    AffCommission,
    CardsSnapshot,
    PostActionResult,
    IpAddress,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Cards {
    Table,
    Id,
    ProductId,
    Content,
    Status,
    OrderId,
    SoldAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AffUsers {
    Table,
    Id,
    Email,
    AffCode,
    Balance,
    TotalEarned,
    TotalWithdrawn,
    WithdrawPasswordHash,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AffLogs {
    Table,
    Id,
    AffUserId,
    OrderId,
    Commission,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Withdrawals {
    Table,
    Id,
    AffUserId,
    Amount,
    Currency,
    Chain,
    WalletAddress,
    Status,
    AdminNote,
    TxHash,
    CreatedAt,
    ProcessedAt,
}

#[derive(DeriveIden)]
enum PaymentConfigs {
    Table,
    Id,
    Channel,
    IsActive,
    ConfigJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SystemSettings {
    Table,
    Key,
    Value,
}
