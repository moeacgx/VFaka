use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add image_url and video_url to products
        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .add_column(ColumnDef::new(Products::ImageUrl).text().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .add_column(ColumnDef::new(Products::VideoUrl).text().null())
                    .to_owned(),
            )
            .await?;

        // Insert default system settings for notifications, storage, and announcements
        let settings: Vec<(&str, &str)> = vec![
            ("announcement_text", ""),
            ("announcement_enabled", "false"),
            ("announcement_type", "info"),
            ("telegram_bot_token", ""),
            ("telegram_chat_id", ""),
            ("telegram_enabled", "false"),
            ("smtp_host", ""),
            ("smtp_port", "465"),
            ("smtp_username", ""),
            ("smtp_password", ""),
            ("smtp_from", ""),
            ("smtp_enabled", "false"),
            ("storage_type", "local"),
            ("s3_endpoint", ""),
            ("s3_bucket", ""),
            ("s3_access_key", ""),
            ("s3_secret_key", ""),
            ("s3_region", "auto"),
        ];

        for (key, value) in settings {
            let sql = format!(
                "INSERT OR IGNORE INTO system_settings (key, value) VALUES ('{}', '{}')",
                key, value
            );
            manager
                .get_connection()
                .execute_unprepared(&sql)
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support DROP COLUMN well, but we can try
        // In practice, down migrations are rarely used
        let keys = [
            "announcement_text", "announcement_enabled", "announcement_type",
            "telegram_bot_token", "telegram_chat_id", "telegram_enabled",
            "smtp_host", "smtp_port", "smtp_username", "smtp_password", "smtp_from", "smtp_enabled",
            "storage_type", "s3_endpoint", "s3_bucket", "s3_access_key", "s3_secret_key", "s3_region",
        ];

        for key in keys {
            let sql = format!("DELETE FROM system_settings WHERE key = '{}'", key);
            manager.get_connection().execute_unprepared(&sql).await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    ImageUrl,
    VideoUrl,
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum SystemSettings {
    Table,
    Key,
    Value,
}
