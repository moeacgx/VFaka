use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260416_000007_product_delivery_mode"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("products"))
                    .add_column(
                        ColumnDef::new(Alias::new("delivery_mode"))
                            .string_len(32)
                            .not_null()
                            .default("card"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("products"))
                    .drop_column(Alias::new("delivery_mode"))
                    .to_owned(),
            )
            .await
    }
}
