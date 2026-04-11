use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::dto::variant_dto::{CreateVariantDto, UpdateVariantDto, VariantResponse};
use aff_entity::entities::{card, product, product_variant};

use super::card_service;

pub async fn list_variants(
    db: &DatabaseConnection,
    product_id: i32,
) -> AppResult<Vec<VariantResponse>> {
    let variants = product_variant::Entity::find()
        .filter(product_variant::Column::ProductId.eq(product_id))
        .order_by_asc(product_variant::Column::SortOrder)
        .order_by_asc(product_variant::Column::Id)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(variants.into_iter().map(to_variant_response).collect())
}

pub async fn get_variant(db: &DatabaseConnection, id: i32) -> AppResult<VariantResponse> {
    let variant = product_variant::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Variant {} not found", id)))?;

    Ok(to_variant_response(variant))
}

pub async fn create_variant(
    db: &DatabaseConnection,
    product_id: i32,
    dto: CreateVariantDto,
) -> AppResult<VariantResponse> {
    // Verify product exists
    product::Entity::find_by_id(product_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", product_id)))?;

    if dto.name.trim().is_empty() {
        return Err(AppError::BadRequest("Variant name cannot be empty".into()));
    }
    if dto.price < 0.0 {
        return Err(AppError::BadRequest("Variant price cannot be negative".into()));
    }

    let now = chrono::Utc::now();
    let model = product_variant::ActiveModel {
        product_id: Set(product_id),
        name: Set(dto.name.trim().to_string()),
        price: Set(dto.price),
        description: Set(dto.description),
        sort_order: Set(dto.sort_order.unwrap_or(0)),
        is_active: Set(dto.is_active.unwrap_or(true)),
        stock_count: Set(0),
        sales_count: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let result = product_variant::Entity::insert(model)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    get_variant(db, result.last_insert_id).await
}

pub async fn update_variant(
    db: &DatabaseConnection,
    id: i32,
    dto: UpdateVariantDto,
) -> AppResult<VariantResponse> {
    let variant = product_variant::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Variant {} not found", id)))?;

    let mut model: product_variant::ActiveModel = variant.into();

    if let Some(name) = dto.name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("Variant name cannot be empty".into()));
        }
        model.name = Set(name.trim().to_string());
    }
    if let Some(price) = dto.price {
        if price < 0.0 {
            return Err(AppError::BadRequest("Variant price cannot be negative".into()));
        }
        model.price = Set(price);
    }
    if let Some(desc) = dto.description {
        model.description = Set(Some(desc));
    }
    if let Some(sort) = dto.sort_order {
        model.sort_order = Set(sort);
    }
    if let Some(active) = dto.is_active {
        model.is_active = Set(active);
    }

    model.updated_at = Set(chrono::Utc::now());
    model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    get_variant(db, id).await
}

pub async fn delete_variant(db: &DatabaseConnection, id: i32) -> AppResult<()> {
    let variant = product_variant::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Variant {} not found", id)))?;

    let product_id = variant.product_id;

    // Check for non-available cards
    let locked_count = card::Entity::find()
        .filter(card::Column::VariantId.eq(id))
        .filter(card::Column::Status.ne("available"))
        .count(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if locked_count > 0 {
        return Err(AppError::Conflict(
            "Cannot delete variant with locked/sold cards".into(),
        ));
    }

    // Delete available cards belonging to this variant
    card::Entity::delete_many()
        .filter(card::Column::VariantId.eq(id))
        .filter(card::Column::Status.eq("available"))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    product_variant::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Sync product stock after deleting variant cards
    card_service::sync_product_stock(db, product_id).await?;

    Ok(())
}

fn to_variant_response(v: product_variant::Model) -> VariantResponse {
    VariantResponse {
        id: v.id,
        product_id: v.product_id,
        name: v.name,
        price: v.price,
        description: v.description,
        sort_order: v.sort_order,
        is_active: v.is_active,
        stock_count: v.stock_count,
        sales_count: v.sales_count,
        created_at: v.created_at,
        updated_at: v.updated_at,
    }
}
