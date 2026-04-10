use sea_orm::*;
use sea_orm::sea_query::Expr;

use aff_common::error::{AppError, AppResult};
use aff_entity::dto::AffWithdrawDto;
use aff_entity::entities::{aff_user, withdrawal};

use super::settings_service;

pub async fn create_withdrawal(
    db: &DatabaseConnection,
    dto: AffWithdrawDto,
) -> AppResult<withdrawal::Model> {
    // Pre-validate outside transaction (read-only checks)
    let user = aff_user::Entity::find()
        .filter(aff_user::Column::Email.eq(&dto.email))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("AFF user not found".into()))?;

    // Verify password
    let hash = user
        .withdraw_password_hash
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Withdraw password not set".into()))?;

    let valid = bcrypt::verify(&dto.password, hash)
        .map_err(|e| AppError::Internal(format!("Password verify failed: {}", e)))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid withdraw password".into()));
    }

    // Check min withdraw amount
    let min_amount = settings_service::get_setting(db, "min_withdraw_amount")
        .await?
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(10.0);

    if dto.amount < min_amount {
        return Err(AppError::BadRequest(format!(
            "Minimum withdrawal amount is {}",
            min_amount
        )));
    }

    // Early balance check (authoritative check is CAS inside txn)
    if user.balance < dto.amount {
        return Err(AppError::BadRequest(format!(
            "Insufficient balance: available {:.2}, requested {:.2}",
            user.balance, dto.amount
        )));
    }

    let user_id = user.id;

    // Transaction: CAS balance deduction + withdrawal insertion
    let txn = db.begin().await.map_err(|e| AppError::Internal(e.to_string()))?;

    // CAS: only deduct if balance >= requested amount (prevents concurrent overdraft)
    let update_result = aff_user::Entity::update_many()
        .col_expr(
            aff_user::Column::Balance,
            Expr::col(aff_user::Column::Balance).sub(dto.amount),
        )
        .col_expr(
            aff_user::Column::TotalWithdrawn,
            Expr::col(aff_user::Column::TotalWithdrawn).add(dto.amount),
        )
        .filter(aff_user::Column::Id.eq(user_id))
        .filter(Expr::col(aff_user::Column::Balance).gte(dto.amount))
        .exec(&txn)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Balance deduction failed: {}", e))
        })?;

    if update_result.rows_affected == 0 {
        txn.rollback().await.ok();
        return Err(AppError::BadRequest(
            "Insufficient balance (concurrent withdrawal detected)".into(),
        ));
    }

    // Insert withdrawal record inside the same transaction
    let now = chrono::Utc::now();
    let model = withdrawal::ActiveModel {
        aff_user_id: Set(user_id),
        amount: Set(dto.amount),
        currency: Set(dto.currency),
        chain: Set(dto.chain),
        wallet_address: Set(dto.wallet_address),
        status: Set("pending".to_string()),
        admin_note: Set(None),
        tx_hash: Set(None),
        created_at: Set(now),
        processed_at: Set(None),
        ..Default::default()
    };

    let withdrawal = withdrawal::Entity::insert(model)
        .exec_with_returning(&txn)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Withdrawal record creation failed: {}", e))
        })?;

    // Commit: both balance deduction and withdrawal record persist atomically
    txn.commit().await.map_err(|e| {
        AppError::Internal(format!("Transaction commit failed: {}", e))
    })?;

    Ok(withdrawal)
}
