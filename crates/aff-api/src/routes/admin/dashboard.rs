use actix_web::{web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, ColumnTrait, QueryOrder};
use std::collections::HashMap;

use aff_common::config::AppConfig;
use aff_common::error::{AppError, AppResult};
use aff_entity::entities::{order, product, card};

pub fn scope() -> actix_web::Scope {
    web::scope("/dashboard")
        .route("", web::get().to(stats))
        .route("/config", web::get().to(admin_config))
}

async fn admin_config(config: web::Data<AppConfig>) -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "allow_command_action": config.security.allow_command_action,
    })))
}

async fn stats(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let today = chrono::Utc::now().date_naive();
    let today_start = today
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::Internal("Failed to compute today start".into()))?
        .and_utc();

    let today_orders = order::Entity::find()
        .filter(order::Column::CreatedAt.gte(today_start))
        .count(db.get_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let today_paid_orders: Vec<order::Model> = order::Entity::find()
        .filter(order::Column::CreatedAt.gte(today_start))
        .filter(order::Column::Status.is_in(["paid", "delivered"]))
        .all(db.get_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let today_income: f64 = today_paid_orders.iter().map(|o| o.total_amount).sum();

    let total_products = product::Entity::find()
        .count(db.get_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Products with stock < 5
    let all_products: Vec<product::Model> = product::Entity::find()
        .filter(product::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let low_stock_count = all_products.iter().filter(|p| p.stock_count < 5).count();

    let low_stock: Vec<serde_json::Value> = all_products
        .into_iter()
        .filter(|p| p.stock_count < 5)
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "stock_count": p.stock_count,
            })
        })
        .collect();

    let total_orders = order::Entity::find()
        .count(db.get_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total_cards = card::Entity::find()
        .filter(card::Column::Status.eq("available"))
        .count(db.get_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Recent 10 orders
    let recent_order_models: Vec<order::Model> = order::Entity::find()
        .order_by_desc(order::Column::CreatedAt)
        .paginate(db.get_ref(), 10)
        .fetch_page(0)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Look up product names for recent orders
    let product_ids: Vec<i32> = recent_order_models.iter().map(|o| o.product_id).collect();
    let products: Vec<product::Model> = product::Entity::find()
        .filter(product::Column::Id.is_in(product_ids))
        .all(db.get_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let product_map: HashMap<i32, String> = products.into_iter().map(|p| (p.id, p.name)).collect();

    let recent_orders: Vec<serde_json::Value> = recent_order_models
        .into_iter()
        .map(|o| {
            let product_name = product_map
                .get(&o.product_id)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "id": o.id,
                "order_no": o.order_no,
                "product_name": product_name,
                "total_amount": o.total_amount,
                "email": o.email,
                "status": o.status,
                "created_at": o.created_at,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "stats": {
            "today_orders": today_orders,
            "today_income": today_income,
            "total_products": total_products,
            "total_orders": total_orders,
            "total_available_cards": total_cards,
            "low_stock_count": low_stock_count,
            "low_stock_alerts": low_stock,
        },
        "recent_orders": recent_orders,
    })))
}
