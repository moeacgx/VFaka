use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::entities::withdrawal;

pub async fn list_withdrawals(
    db: &DatabaseConnection,
    status_filter: Option<String>,
) -> AppResult<Vec<withdrawal::Model>> {
    let mut query = withdrawal::Entity::find().order_by_desc(withdrawal::Column::CreatedAt);

    if let Some(status) = status_filter {
        query = query.filter(withdrawal::Column::Status.eq(status));
    }

    query
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn approve_withdrawal(db: &DatabaseConnection, id: i32) -> AppResult<withdrawal::Model> {
    let existing = withdrawal::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Withdrawal {} not found", id)))?;

    if existing.status != "pending" {
        return Err(AppError::Conflict(format!(
            "Withdrawal {} status is '{}', can only approve 'pending'",
            id, existing.status
        )));
    }

    let mut model: withdrawal::ActiveModel = existing.into();
    model.status = Set("approved".to_string());
    model.processed_at = Set(Some(chrono::Utc::now()));

    model
        .update(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn reject_withdrawal(
    db: &DatabaseConnection,
    id: i32,
    note: &str,
) -> AppResult<withdrawal::Model> {
    let existing = withdrawal::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Withdrawal {} not found", id)))?;

    if existing.status != "pending" {
        return Err(AppError::Conflict(format!(
            "Withdrawal {} status is '{}', can only reject 'pending'",
            id, existing.status
        )));
    }

    // Compute refund values from the Model BEFORE converting to ActiveModel
    let aff_user = aff_entity::entities::aff_user::Entity::find_by_id(existing.aff_user_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("AFF user not found".to_string()))?;

    let new_balance = aff_user.balance + existing.amount;
    let new_withdrawn = (aff_user.total_withdrawn - existing.amount).max(0.0);

    // Transaction: refund balance + update withdrawal status atomically
    let txn = db.begin().await.map_err(|e| AppError::Internal(e.to_string()))?;

    let mut aff_model: aff_entity::entities::aff_user::ActiveModel = aff_user.into();
    aff_model.balance = Set(new_balance);
    aff_model.total_withdrawn = Set(new_withdrawn);
    aff_model
        .update(&txn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut model: withdrawal::ActiveModel = existing.into();
    model.status = Set("rejected".to_string());
    model.admin_note = Set(Some(note.to_string()));
    model.processed_at = Set(Some(chrono::Utc::now()));

    let result = model
        .update(&txn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    txn.commit().await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result)
}

pub async fn complete_withdrawal(
    db: &DatabaseConnection,
    id: i32,
    tx_hash: &str,
) -> AppResult<withdrawal::Model> {
    let existing = withdrawal::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Withdrawal {} not found", id)))?;

    if existing.status != "approved" {
        return Err(AppError::Conflict(format!(
            "Withdrawal {} status is '{}', can only complete 'approved'",
            id, existing.status
        )));
    }

    let mut model: withdrawal::ActiveModel = existing.into();
    model.status = Set("completed".to_string());
    model.tx_hash = Set(Some(tx_hash.to_string()));
    model.processed_at = Set(Some(chrono::Utc::now()));

    model
        .update(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
