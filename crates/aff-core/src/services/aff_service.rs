use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_common::id_gen::generate_aff_code;
use aff_entity::dto::{AffQueryResponse, AffRegisterDto};
use aff_entity::entities::{aff_log, aff_user, order, product};

pub async fn list_aff_users(db: &DatabaseConnection) -> AppResult<Vec<aff_user::Model>> {
    aff_user::Entity::find()
        .order_by_desc(aff_user::Column::CreatedAt)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn register(
    db: &DatabaseConnection,
    dto: AffRegisterDto,
) -> AppResult<aff_user::Model> {
    let existing = aff_user::Entity::find()
        .filter(aff_user::Column::Email.eq(&dto.email))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered as AFF user".into()));
    }

    let password_hash = bcrypt::hash(&dto.withdraw_password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Password hash failed: {}", e)))?;

    let aff_code = generate_aff_code();

    let now = chrono::Utc::now();
    let model = aff_user::ActiveModel {
        email: Set(dto.email),
        aff_code: Set(aff_code),
        balance: Set(0.0),
        total_earned: Set(0.0),
        total_withdrawn: Set(0.0),
        withdraw_password_hash: Set(Some(password_hash)),
        created_at: Set(now),
        ..Default::default()
    };

    aff_user::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn query_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> AppResult<AffQueryResponse> {
    let user = aff_user::Entity::find()
        .filter(aff_user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("AFF user not found".into()))?;

    Ok(AffQueryResponse {
        email: user.email,
        aff_code: user.aff_code,
        balance: user.balance,
        total_earned: user.total_earned,
        total_withdrawn: user.total_withdrawn,
        created_at: user.created_at,
    })
}

pub async fn get_user_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> AppResult<Option<aff_user::Model>> {
    aff_user::Entity::find()
        .filter(aff_user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn get_user_by_code(
    db: &DatabaseConnection,
    aff_code: &str,
) -> AppResult<Option<aff_user::Model>> {
    aff_user::Entity::find()
        .filter(aff_user::Column::AffCode.eq(aff_code))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn process_commission(
    db: &DatabaseConnection,
    order: &order::Model,
) -> AppResult<()> {
    let aff_code = match &order.aff_code {
        Some(code) if !code.is_empty() => code.clone(),
        _ => return Ok(()),
    };

    let aff_user = match get_user_by_code(db, &aff_code).await? {
        Some(u) => u,
        None => return Ok(()),
    };

    // Don't give commission to self
    if aff_user.email == order.email {
        return Ok(());
    }

    let prod = product::Entity::find_by_id(order.product_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rate = prod
        .and_then(|p| p.aff_commission_rate)
        .unwrap_or(0.0);

    if rate <= 0.0 {
        return Ok(());
    }

    let commission = order.total_amount * rate;
    if commission <= 0.0 {
        return Ok(());
    }

    // Create aff_log
    let now = chrono::Utc::now();
    let log_model = aff_log::ActiveModel {
        aff_user_id: Set(aff_user.id),
        order_id: Set(order.id),
        commission: Set(commission),
        status: Set("credited".to_string()),
        created_at: Set(now),
        ..Default::default()
    };
    aff_log::Entity::insert(log_model)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Update aff_user balance
    let new_balance = aff_user.balance + commission;
    let new_earned = aff_user.total_earned + commission;
    let aff_email = aff_user.email.clone();
    let mut user_model: aff_user::ActiveModel = aff_user.into();
    user_model.balance = Set(new_balance);
    user_model.total_earned = Set(new_earned);
    user_model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    // Update order aff_commission
    let o = order::Entity::find_by_id(order.id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Internal("Order not found for commission update".into()))?;
    let mut order_model: order::ActiveModel = o.into();
    order_model.aff_commission = Set(commission);
    order_model.aff_user_email = Set(Some(aff_email));
    order_model.updated_at = Set(chrono::Utc::now());
    order_model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn get_logs(
    db: &DatabaseConnection,
    email: &str,
) -> AppResult<Vec<aff_log::Model>> {
    let user = aff_user::Entity::find()
        .filter(aff_user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("AFF user not found".into()))?;

    aff_log::Entity::find()
        .filter(aff_log::Column::AffUserId.eq(user.id))
        .order_by_desc(aff_log::Column::CreatedAt)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
