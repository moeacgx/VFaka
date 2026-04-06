use sea_orm::*;
use sea_orm::prelude::Expr;

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

    // Set products in this category to uncategorized (null) instead of blocking
    product::Entity::update_many()
        .col_expr(product::Column::CategoryId, Expr::value(Option::<i32>::None))
        .filter(product::Column::CategoryId.eq(id))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    category::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn batch_delete_categories(db: &DatabaseConnection, ids: Vec<i32>) -> AppResult<u64> {
    // Unlink products from these categories
    product::Entity::update_many()
        .col_expr(product::Column::CategoryId, Expr::value(Option::<i32>::None))
        .filter(product::Column::CategoryId.is_in(ids.clone()))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let result = category::Entity::delete_many()
        .filter(category::Column::Id.is_in(ids))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result.rows_affected)
}
