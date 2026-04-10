use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::dto::{CreateCouponDto, UpdateCouponDto, ValidateCouponResponse};
use aff_entity::entities::coupon;

/// Validate a coupon and return the discount amount.
pub async fn validate_coupon(
    db: &DatabaseConnection,
    code: &str,
    product_id: i32,
    subtotal: f64,
) -> AppResult<ValidateCouponResponse> {
    let coupon = coupon::Entity::find()
        .filter(coupon::Column::Code.eq(code))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let coupon = match coupon {
        Some(c) => c,
        None => {
            return Ok(ValidateCouponResponse {
                valid: false,
                discount_type: None,
                discount_value: None,
                discount_amount: None,
                message: Some("优惠码不存在".to_string()),
            });
        }
    };

    if !coupon.is_active {
        return Ok(ValidateCouponResponse {
            valid: false,
            discount_type: None,
            discount_value: None,
            discount_amount: None,
            message: Some("优惠码已停用".to_string()),
        });
    }

    let now = chrono::Utc::now();
    if let Some(from) = coupon.valid_from {
        if now < from {
            return Ok(ValidateCouponResponse {
                valid: false,
                discount_type: None,
                discount_value: None,
                discount_amount: None,
                message: Some("优惠码尚未生效".to_string()),
            });
        }
    }
    if let Some(to) = coupon.valid_to {
        if now > to {
            return Ok(ValidateCouponResponse {
                valid: false,
                discount_type: None,
                discount_value: None,
                discount_amount: None,
                message: Some("优惠码已过期".to_string()),
            });
        }
    }

    if let Some(max) = coupon.max_uses {
        if coupon.used_count >= max {
            return Ok(ValidateCouponResponse {
                valid: false,
                discount_type: None,
                discount_value: None,
                discount_amount: None,
                message: Some("优惠码已达最大使用次数".to_string()),
            });
        }
    }

    // Check product scope
    if let Some(pid) = coupon.product_id {
        if pid != product_id {
            return Ok(ValidateCouponResponse {
                valid: false,
                discount_type: None,
                discount_value: None,
                discount_amount: None,
                message: Some("优惠码不适用于该商品".to_string()),
            });
        }
    }

    if subtotal < coupon.min_amount {
        return Ok(ValidateCouponResponse {
            valid: false,
            discount_type: Some(coupon.discount_type.clone()),
            discount_value: Some(coupon.discount_value),
            discount_amount: None,
            message: Some(format!("最低消费 ¥{:.2}", coupon.min_amount)),
        });
    }

    let discount = calc_discount(&coupon.discount_type, coupon.discount_value, subtotal);

    Ok(ValidateCouponResponse {
        valid: true,
        discount_type: Some(coupon.discount_type),
        discount_value: Some(coupon.discount_value),
        discount_amount: Some(discount),
        message: None,
    })
}

/// Calculate the discount amount. Ensures discount never exceeds subtotal.
pub fn calc_discount(discount_type: &str, discount_value: f64, subtotal: f64) -> f64 {
    let raw = match discount_type {
        "percentage" => subtotal * discount_value / 100.0,
        "fixed" => discount_value,
        _ => 0.0,
    };
    // Round to 2 decimals, never exceed subtotal, never negative
    let d = (raw * 100.0).floor() / 100.0;
    d.min(subtotal).max(0.0)
}

/// Increment used_count via CAS to avoid race conditions.
pub async fn use_coupon(db: &DatabaseConnection, code: &str) -> AppResult<()> {
    use sea_orm::sea_query::Expr;

    let result = coupon::Entity::update_many()
        .col_expr(
            coupon::Column::UsedCount,
            Expr::col(coupon::Column::UsedCount).add(1),
        )
        .filter(coupon::Column::Code.eq(code))
        .filter(coupon::Column::IsActive.eq(true))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if result.rows_affected == 0 {
        return Err(AppError::BadRequest("Coupon not found or inactive".to_string()));
    }
    Ok(())
}

// --- Admin CRUD ---

pub async fn list_coupons(
    db: &DatabaseConnection,
    page: u64,
    per_page: u64,
    product_id_filter: Option<i32>,
    is_active_filter: Option<bool>,
) -> AppResult<(Vec<coupon::Model>, u64)> {
    let mut query = coupon::Entity::find().order_by_desc(coupon::Column::CreatedAt);

    if let Some(pid) = product_id_filter {
        query = query.filter(coupon::Column::ProductId.eq(pid));
    }
    if let Some(active) = is_active_filter {
        query = query.filter(coupon::Column::IsActive.eq(active));
    }

    let paginator = query.paginate(db, per_page);
    let total = paginator
        .num_items()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let items = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((items, total))
}

pub async fn create_coupon(
    db: &DatabaseConnection,
    dto: CreateCouponDto,
) -> AppResult<coupon::Model> {
    if dto.discount_type != "fixed" && dto.discount_type != "percentage" {
        return Err(AppError::BadRequest(
            "discount_type must be 'fixed' or 'percentage'".to_string(),
        ));
    }
    if dto.discount_value <= 0.0 {
        return Err(AppError::BadRequest(
            "discount_value must be positive".to_string(),
        ));
    }
    if dto.discount_type == "percentage" && dto.discount_value > 100.0 {
        return Err(AppError::BadRequest(
            "percentage discount cannot exceed 100".to_string(),
        ));
    }

    let model = coupon::ActiveModel {
        code: Set(dto.code.trim().to_uppercase()),
        discount_type: Set(dto.discount_type),
        discount_value: Set(dto.discount_value),
        product_id: Set(dto.product_id),
        min_amount: Set(dto.min_amount.unwrap_or(0.0)),
        max_uses: Set(dto.max_uses),
        used_count: Set(0),
        valid_from: Set(dto.valid_from),
        valid_to: Set(dto.valid_to),
        is_active: Set(dto.is_active.unwrap_or(true)),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    coupon::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_coupon(
    db: &DatabaseConnection,
    id: i32,
    dto: UpdateCouponDto,
) -> AppResult<coupon::Model> {
    let existing = coupon::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Coupon {} not found", id)))?;

    let mut model: coupon::ActiveModel = existing.into();

    if let Some(code) = dto.code {
        model.code = Set(code.trim().to_uppercase());
    }
    if let Some(dt) = &dto.discount_type {
        if dt != "fixed" && dt != "percentage" {
            return Err(AppError::BadRequest(
                "discount_type must be 'fixed' or 'percentage'".to_string(),
            ));
        }
        model.discount_type = Set(dt.clone());
    }
    if let Some(dv) = dto.discount_value {
        if dv <= 0.0 {
            return Err(AppError::BadRequest(
                "discount_value must be positive".to_string(),
            ));
        }
        model.discount_value = Set(dv);
    }
    if let Some(pid) = dto.product_id {
        model.product_id = Set(pid);
    }
    if let Some(ma) = dto.min_amount {
        model.min_amount = Set(ma);
    }
    if let Some(mu) = dto.max_uses {
        model.max_uses = Set(mu);
    }
    if let Some(vf) = dto.valid_from {
        model.valid_from = Set(vf);
    }
    if let Some(vt) = dto.valid_to {
        model.valid_to = Set(vt);
    }
    if let Some(active) = dto.is_active {
        model.is_active = Set(active);
    }

    model
        .update(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn delete_coupon(db: &DatabaseConnection, id: i32) -> AppResult<()> {
    let result = coupon::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound(format!("Coupon {} not found", id)));
    }
    Ok(())
}

pub async fn batch_delete_coupons(db: &DatabaseConnection, ids: Vec<i32>) -> AppResult<u64> {
    let result = coupon::Entity::delete_many()
        .filter(coupon::Column::Id.is_in(ids))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result.rows_affected)
}
