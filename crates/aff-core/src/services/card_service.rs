use sea_orm::prelude::Expr;
use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::entities::{card, product};

pub async fn list_cards(
    db: &DatabaseConnection,
    product_id: Option<i32>,
    status: Option<String>,
) -> AppResult<Vec<card::Model>> {
    let mut query = card::Entity::find().order_by_desc(card::Column::CreatedAt);

    if let Some(pid) = product_id {
        query = query.filter(card::Column::ProductId.eq(pid));
    }
    if let Some(s) = status {
        query = query.filter(card::Column::Status.eq(s));
    }

    query
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn import_cards(
    db: &DatabaseConnection,
    product_id: i32,
    text: &str,
) -> AppResult<u64> {
    // Verify product exists
    product::Entity::find_by_id(product_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", product_id)))?;

    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return Err(AppError::BadRequest("No cards to import".into()));
    }

    let now = chrono::Utc::now();
    let models: Vec<card::ActiveModel> = lines
        .iter()
        .map(|line| card::ActiveModel {
            product_id: Set(product_id),
            content: Set(ToString::to_string(line)),
            status: Set("available".to_string()),
            order_id: Set(None),
            sold_at: Set(None),
            created_at: Set(now),
            ..Default::default()
        })
        .collect();

    let count = models.len() as u64;

    card::Entity::insert_many(models)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Update product stock_count
    let available_count = card::Entity::find()
        .filter(card::Column::ProductId.eq(product_id))
        .filter(card::Column::Status.eq("available"))
        .count(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let prod = product::Entity::find_by_id(product_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", product_id)))?;

    let mut prod_model: product::ActiveModel = prod.into();
    prod_model.stock_count = Set(available_count as i32);
    prod_model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(count)
}

pub async fn delete_card(db: &DatabaseConnection, id: i32) -> AppResult<()> {
    let card_model = card::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Card {} not found", id)))?;

    if card_model.status != "available" {
        return Err(AppError::Conflict(format!(
            "Card {} status is '{}', only 'available' cards can be deleted",
            id, card_model.status
        )));
    }

    let product_id = card_model.product_id;

    card::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Update product stock_count
    sync_stock_count(db, product_id).await?;

    Ok(())
}

pub async fn lock_cards(
    db: &DatabaseConnection,
    product_id: i32,
    quantity: i32,
    order_id: i32,
) -> AppResult<Vec<card::Model>> {
    let cards = card::Entity::find()
        .filter(card::Column::ProductId.eq(product_id))
        .filter(card::Column::Status.eq("available"))
        .order_by_asc(card::Column::Id)
        .limit(quantity as u64)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if (cards.len() as i32) < quantity {
        return Err(AppError::Conflict(format!(
            "Not enough cards: requested {}, available {}",
            quantity,
            cards.len()
        )));
    }

    let card_ids: Vec<i32> = cards.iter().map(|c| c.id).collect();

    card::Entity::update_many()
        .col_expr(card::Column::Status, Expr::value("locked"))
        .col_expr(card::Column::OrderId, Expr::value(Some(order_id)))
        .filter(card::Column::Id.is_in(card_ids.clone()))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Re-fetch the updated cards
    let updated_cards = card::Entity::find()
        .filter(card::Column::Id.is_in(card_ids))
        .order_by_asc(card::Column::Id)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    sync_stock_count(db, product_id).await?;

    Ok(updated_cards)
}

pub async fn release_cards(db: &DatabaseConnection, card_ids: &[i32]) -> AppResult<()> {
    if card_ids.is_empty() {
        return Ok(());
    }

    // Get product_ids for stock sync
    let cards = card::Entity::find()
        .filter(card::Column::Id.is_in(card_ids.to_vec()))
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let product_ids: Vec<i32> = cards.iter().map(|c| c.product_id).collect::<std::collections::HashSet<_>>().into_iter().collect();

    card::Entity::update_many()
        .col_expr(card::Column::Status, Expr::value("available"))
        .filter(card::Column::Id.is_in(card_ids.to_vec()))
        .filter(card::Column::Status.eq("locked"))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    for pid in product_ids {
        sync_stock_count(db, pid).await?;
    }

    Ok(())
}

pub async fn deliver_cards(
    db: &DatabaseConnection,
    card_ids: &[i32],
    order_id: i32,
) -> AppResult<()> {
    if card_ids.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now();

    card::Entity::update_many()
        .col_expr(card::Column::Status, Expr::value("sold"))
        .col_expr(card::Column::OrderId, Expr::value(Some(order_id)))
        .col_expr(card::Column::SoldAt, Expr::value(Some(now)))
        .filter(card::Column::Id.is_in(card_ids.to_vec()))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

async fn sync_stock_count(db: &DatabaseConnection, product_id: i32) -> AppResult<()> {
    let available_count = card::Entity::find()
        .filter(card::Column::ProductId.eq(product_id))
        .filter(card::Column::Status.eq("available"))
        .count(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let prod = product::Entity::find_by_id(product_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(p) = prod {
        let mut model: product::ActiveModel = p.into();
        model.stock_count = Set(available_count as i32);
        model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(())
}
