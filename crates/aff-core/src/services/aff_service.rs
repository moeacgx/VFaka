use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_common::id_gen::generate_aff_code;
use aff_entity::dto::{AffNextLevel, AffQueryResponse, AffRegisterDto, CreateAffTierDto, UpdateAffTierDto};
use aff_entity::entities::{aff_log, aff_tier, aff_user, order, product};

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
        level: Set(1),
        created_at: Set(now),
        ..Default::default()
    };

    aff_user::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn query_by_code(
    db: &DatabaseConnection,
    aff_code: &str,
) -> AppResult<AffQueryResponse> {
    let user = aff_user::Entity::find()
        .filter(aff_user::Column::AffCode.eq(aff_code))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("AFF user not found".into()))?;

    build_query_response(db, user).await
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

    build_query_response(db, user).await
}

async fn build_query_response(
    db: &DatabaseConnection,
    user: aff_user::Model,
) -> AppResult<AffQueryResponse> {

    let tiers = list_tiers(db).await?;

    let current_tier = tiers.iter().find(|t| t.level == user.level);
    let (level_name, commission_rate) = match current_tier {
        Some(t) => (t.name.clone(), t.commission_rate),
        None => ("未知".to_string(), 0.05),
    };

    // Find next tier
    let next_level = tiers
        .iter()
        .filter(|t| t.level > user.level)
        .min_by_key(|t| t.level)
        .map(|t| AffNextLevel {
            level: t.level,
            name: t.name.clone(),
            commission_rate: t.commission_rate,
            required_amount: t.required_amount,
            remaining: (t.required_amount - user.total_earned).max(0.0),
        });

    Ok(AffQueryResponse {
        email: user.email,
        aff_code: user.aff_code,
        balance: user.balance,
        total_earned: user.total_earned,
        total_withdrawn: user.total_withdrawn,
        level: user.level,
        level_name,
        commission_rate,
        next_level,
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

/// Verify an AFF user's withdrawal password. Returns the user on success.
pub async fn verify_user_password(
    db: &DatabaseConnection,
    aff_code: &str,
    password: &str,
) -> AppResult<aff_user::Model> {
    let user = get_user_by_code(db, aff_code)
        .await?
        .ok_or_else(|| AppError::NotFound("AFF user not found".into()))?;
    let hash = user.withdraw_password_hash.as_deref()
        .ok_or_else(|| AppError::BadRequest("Withdraw password not set".into()))?;
    let valid = bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(format!("Password verify failed: {}", e)))?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid password".into()));
    }
    Ok(user)
}

/// Get the commission rate for a user based on their tier level.
/// Falls back to the product-specific rate, then to the user's tier rate.
async fn get_commission_rate(
    db: &DatabaseConnection,
    user: &aff_user::Model,
    product_rate: Option<f64>,
) -> f64 {
    // Product-specific override takes precedence
    if let Some(rate) = product_rate {
        if rate > 0.0 {
            return rate;
        }
    }

    // Use tier-based rate
    let tier = aff_tier::Entity::find()
        .filter(aff_tier::Column::Level.eq(user.level))
        .one(db)
        .await
        .ok()
        .flatten();

    match tier {
        Some(t) => t.commission_rate,
        None => 0.05, // fallback
    }
}

/// Check if user should be upgraded to a higher tier based on total_earned.
async fn maybe_upgrade_tier(
    db: &DatabaseConnection,
    user: &aff_user::Model,
    new_total_earned: f64,
) -> AppResult<Option<i32>> {
    let tiers = list_tiers(db).await?;

    // Find the highest tier the user qualifies for
    let best_tier = tiers
        .iter()
        .filter(|t| new_total_earned >= t.required_amount)
        .max_by_key(|t| t.level);

    match best_tier {
        Some(t) if t.level > user.level => {
            tracing::info!(
                "AFF user {} upgraded: level {} → {} ({})",
                user.email, user.level, t.level, t.name
            );
            Ok(Some(t.level))
        }
        _ => Ok(None),
    }
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

    // Idempotency: skip if commission already recorded for this order
    let existing_log = aff_log::Entity::find()
        .filter(aff_log::Column::OrderId.eq(order.id))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if existing_log.is_some() {
        tracing::info!(
            order_id = order.id,
            "Commission already recorded for this order, skipping"
        );
        return Ok(());
    }

    let prod = product::Entity::find_by_id(order.product_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let product_rate = prod.and_then(|p| p.aff_commission_rate);
    let rate = get_commission_rate(db, &aff_user, product_rate).await;

    if rate <= 0.0 {
        return Ok(());
    }

    let commission = crate::round_money(order.total_amount * rate);
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

    // Update aff_user balance + total_earned atomically
    use sea_orm::sea_query::Expr;
    let aff_email = aff_user.email.clone();
    let aff_user_id = aff_user.id;

    aff_user::Entity::update_many()
        .col_expr(
            aff_user::Column::Balance,
            Expr::col(aff_user::Column::Balance).add(commission),
        )
        .col_expr(
            aff_user::Column::TotalEarned,
            Expr::col(aff_user::Column::TotalEarned).add(commission),
        )
        .filter(aff_user::Column::Id.eq(aff_user_id))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Re-read user to check tier upgrade with actual total_earned
    if let Some(updated_user) = aff_user::Entity::find_by_id(aff_user_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if let Some(new_level) = maybe_upgrade_tier(db, &updated_user, updated_user.total_earned).await? {
            let mut user_model: aff_user::ActiveModel = updated_user.into();
            user_model.level = Set(new_level);
            user_model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

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

pub async fn get_logs_by_code(
    db: &DatabaseConnection,
    aff_code: &str,
) -> AppResult<Vec<aff_log::Model>> {
    let user = aff_user::Entity::find()
        .filter(aff_user::Column::AffCode.eq(aff_code))
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

// ===== Tier Management =====

pub async fn list_tiers(db: &DatabaseConnection) -> AppResult<Vec<aff_tier::Model>> {
    aff_tier::Entity::find()
        .order_by_asc(aff_tier::Column::Level)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn create_tier(
    db: &DatabaseConnection,
    dto: CreateAffTierDto,
) -> AppResult<aff_tier::Model> {
    let model = aff_tier::ActiveModel {
        level: Set(dto.level),
        name: Set(dto.name),
        commission_rate: Set(dto.commission_rate),
        required_amount: Set(dto.required_amount),
        ..Default::default()
    };
    aff_tier::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_tier(
    db: &DatabaseConnection,
    level: i32,
    dto: UpdateAffTierDto,
) -> AppResult<aff_tier::Model> {
    let tier = aff_tier::Entity::find()
        .filter(aff_tier::Column::Level.eq(level))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Tier level {} not found", level)))?;

    let mut model: aff_tier::ActiveModel = tier.into();
    if let Some(name) = dto.name {
        model.name = Set(name);
    }
    if let Some(rate) = dto.commission_rate {
        model.commission_rate = Set(rate);
    }
    if let Some(amount) = dto.required_amount {
        model.required_amount = Set(amount);
    }

    model
        .update(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn delete_tier(db: &DatabaseConnection, level: i32) -> AppResult<()> {
    // Don't allow deleting level 1 (base tier)
    if level <= 1 {
        return Err(AppError::BadRequest("Cannot delete the base tier".into()));
    }

    // Downgrade users at this level to level 1
    let users_at_level = aff_user::Entity::find()
        .filter(aff_user::Column::Level.eq(level))
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    for u in users_at_level {
        let mut am: aff_user::ActiveModel = u.into();
        am.level = Set(1);
        am.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    }

    aff_tier::Entity::delete_many()
        .filter(aff_tier::Column::Level.eq(level))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}
