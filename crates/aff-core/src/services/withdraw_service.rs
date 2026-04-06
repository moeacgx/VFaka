use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::dto::AffWithdrawDto;
use aff_entity::entities::{aff_user, withdrawal};

use super::settings_service;

pub async fn create_withdrawal(
    db: &DatabaseConnection,
    dto: AffWithdrawDto,
) -> AppResult<withdrawal::Model> {
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

    // Check balance
    if user.balance < dto.amount {
        return Err(AppError::BadRequest(format!(
            "Insufficient balance: available {:.2}, requested {:.2}",
            user.balance, dto.amount
        )));
    }

    // Deduct balance
    let new_balance = user.balance - dto.amount;
    let new_withdrawn = user.total_withdrawn + dto.amount;
    let user_id = user.id;
    let mut user_model: aff_user::ActiveModel = user.into();
    user_model.balance = Set(new_balance);
    user_model.total_withdrawn = Set(new_withdrawn);
    user_model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    // Create withdrawal
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

    withdrawal::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
