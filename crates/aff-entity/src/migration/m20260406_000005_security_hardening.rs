use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260406_000005_security_hardening"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add query_token for secure public order queries
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("orders"))
                    .add_column(
                        ColumnDef::new(Alias::new("query_token"))
                            .string_len(32)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add post_action_status for retryable post-pay action tracking
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("orders"))
                    .add_column(
                        ColumnDef::new(Alias::new("post_action_status"))
                            .string_len(20)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support DROP COLUMN easily
        Ok(())
    }
}
