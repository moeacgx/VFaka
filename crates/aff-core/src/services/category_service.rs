use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::dto::{CreateCategoryDto, UpdateCategoryDto};
use aff_entity::entities::{category, product};

pub async fn list_categories(db: &DatabaseConnection) -> AppResult<Vec<category::Model>> {
    category::Entity::find()
        .order_by_asc(category::Column::SortOrder)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn get_category(db: &DatabaseConnection, id: i32) -> AppResult<category::Model> {
    category::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Category {} not found", id)))
}

pub async fn create_category(
    db: &DatabaseConnection,
    dto: CreateCategoryDto,
) -> AppResult<category::Model> {
    let model = category::ActiveModel {
        name: Set(dto.name),
        sort_order: Set(dto.sort_order.unwrap_or(0)),
        is_active: Set(true),
        ..Default::default()
    };
    category::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_category(
    db: &DatabaseConnection,
    id: i32,
    dto: UpdateCategoryDto,
) -> AppResult<category::Model> {
    let existing = get_category(db, id).await?;
    let mut model: category::ActiveModel = existing.into();

    if let Some(name) = dto.name {
        model.name = Set(name);
    }
    if let Some(sort_order) = dto.sort_order {
        model.sort_order = Set(sort_order);
    }
    if let Some(is_active) = dto.is_active {
        model.is_active = Set(is_active);
    }

    model
        .update(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn delete_category(db: &DatabaseConnection, id: i32) -> AppResult<()> {
    let _existing = get_category(db, id).await?;

    let product_count = product::Entity::find()
        .filter(product::Column::CategoryId.eq(id))
        .count(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if product_count > 0 {
        return Err(AppError::Conflict(format!(
            "Category {} has {} products, cannot delete",
            id, product_count
        )));
    }

    category::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}
