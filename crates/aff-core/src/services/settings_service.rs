use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::entities::system_setting;

pub async fn get_setting(db: &DatabaseConnection, key: &str) -> AppResult<Option<String>> {
    let result = system_setting::Entity::find_by_id(key.to_string())
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result.map(|m| m.value))
}

pub async fn get_all_settings(
    db: &DatabaseConnection,
) -> AppResult<Vec<system_setting::Model>> {
    system_setting::Entity::find()
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn set_setting(
    db: &DatabaseConnection,
    key: &str,
    value: &str,
) -> AppResult<system_setting::Model> {
    let existing = system_setting::Entity::find_by_id(key.to_string())
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match existing {
        Some(_) => {
            let model = system_setting::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
            };
            model
                .update(db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))
        }
        None => {
            let model = system_setting::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
            };
            system_setting::Entity::insert(model)
                .exec_with_returning(db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))
        }
    }
}
