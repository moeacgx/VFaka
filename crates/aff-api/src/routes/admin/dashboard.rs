use actix_web::{web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, ColumnTrait};

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

    let today_revenue: f64 = today_paid_orders.iter().map(|o| o.total_amount).sum();

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

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "today_orders": today_orders,
        "today_revenue": today_revenue,
        "total_products": total_products,
        "total_orders": total_orders,
        "total_available_cards": total_cards,
        "low_stock_alerts": low_stock,
    })))
}
