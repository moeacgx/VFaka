use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::entities::payment_config;

pub async fn list_configs(db: &DatabaseConnection) -> AppResult<Vec<payment_config::Model>> {
    payment_config::Entity::find()
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_config(
    db: &DatabaseConnection,
    channel: &str,
    config_json: &str,
    is_active: bool,
) -> AppResult<payment_config::Model> {
    let existing = payment_config::Entity::find()
        .filter(payment_config::Column::Channel.eq(channel))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Payment config for channel '{}' not found", channel)))?;

    let mut model: payment_config::ActiveModel = existing.into();
    model.config_json = Set(config_json.to_string());
    model.is_active = Set(is_active);
    model.updated_at = Set(chrono::Utc::now());

    model
        .update(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
