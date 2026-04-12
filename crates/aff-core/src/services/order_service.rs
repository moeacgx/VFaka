use sea_orm::*;
use std::collections::HashMap;

use aff_common::error::{AppError, AppResult};
use aff_common::id_gen::generate_query_token;
use aff_entity::dto::OrderResponse;
use aff_entity::entities::{order, product};

pub async fn create_order(
    db: &DatabaseConnection,
    order_no: String,
    product_id: i32,
    quantity: i32,
    total_amount: f64,
    email: String,
    payment_method: String,
    payment_channel: String,
    aff_code: Option<String>,
    aff_user_email: Option<String>,
    ip_address: Option<String>,
    coupon_code: Option<String>,
    discount_amount: f64,
    variant_id: Option<i32>,
    variant_name: Option<String>,
) -> AppResult<order::Model> {
    let now = chrono::Utc::now();
    let query_token = generate_query_token();
    let model = order::ActiveModel {
        order_no: Set(order_no),
        product_id: Set(product_id),
        variant_id: Set(variant_id),
        variant_name: Set(variant_name),
        quantity: Set(quantity),
        total_amount: Set(total_amount),
        email: Set(email),
        payment_method: Set(payment_method),
        payment_channel: Set(payment_channel),
        status: Set("pending".to_string()),
        trade_no: Set(None),
        pay_time: Set(None),
        aff_code: Set(aff_code),
        aff_user_email: Set(aff_user_email),
        aff_commission: Set(0.0),
        cards_snapshot: Set(None),
        post_action_result: Set(None),
        post_action_status: Set(None),
        ip_address: Set(ip_address),
        coupon_code: Set(coupon_code),
        discount_amount: Set(discount_amount),
        query_token: Set(Some(query_token)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    order::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn get_order_by_no(
    db: &DatabaseConnection,
    order_no: &str,
) -> AppResult<Option<order::Model>> {
    order::Entity::find()
        .filter(order::Column::OrderNo.eq(order_no))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn list_orders_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> AppResult<Vec<order::Model>> {
    order::Entity::find()
        .filter(order::Column::Email.eq(email))
        .order_by_desc(order::Column::CreatedAt)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn count_pending_orders_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> AppResult<u64> {
    order::Entity::find()
        .filter(order::Column::Email.eq(email))
        .filter(order::Column::Status.eq("pending"))
        .count(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_order_status(
    db: &DatabaseConnection,
    order_no: &str,
    status: &str,
) -> AppResult<()> {
    let o = order::Entity::find()
        .filter(order::Column::OrderNo.eq(order_no))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_no)))?;

    let mut model: order::ActiveModel = o.into();
    model.status = Set(status.to_string());
    model.updated_at = Set(chrono::Utc::now());
    model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

/// Atomically claim an order for processing via CAS.
/// Returns true if this caller won the claim, false if another thread already claimed it.
pub async fn claim_order_for_processing(
    db: &DatabaseConnection,
    order_no: &str,
) -> AppResult<bool> {
    use sea_orm::sea_query::Expr;

    let result = order::Entity::update_many()
        .col_expr(order::Column::Status, Expr::value("processing"))
        .col_expr(order::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
        .filter(order::Column::OrderNo.eq(order_no))
        .filter(order::Column::Status.eq("paid"))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result.rows_affected == 1)
}

/// Atomically transition order from pending to paid via CAS.
/// Returns true if transition happened, false if order was not in pending state.
pub async fn mark_order_paid(
    db: &DatabaseConnection,
    order_no: &str,
) -> AppResult<bool> {
    use sea_orm::sea_query::Expr;

    let result = order::Entity::update_many()
        .col_expr(order::Column::Status, Expr::value("paid"))
        .col_expr(order::Column::PayTime, Expr::value(Some(chrono::Utc::now())))
        .col_expr(order::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
        .filter(order::Column::OrderNo.eq(order_no))
        .filter(order::Column::Status.eq("pending"))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result.rows_affected == 1)
}

pub async fn update_order_trade_no(
    db: &DatabaseConnection,
    order_no: &str,
    trade_no: &str,
) -> AppResult<()> {
    let o = order::Entity::find()
        .filter(order::Column::OrderNo.eq(order_no))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_no)))?;

    let mut model: order::ActiveModel = o.into();
    model.trade_no = Set(Some(trade_no.to_string()));
    model.updated_at = Set(chrono::Utc::now());
    model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn deliver_order(
    db: &DatabaseConnection,
    order_no: &str,
    cards_snapshot: &str,
) -> AppResult<()> {
    let o = order::Entity::find()
        .filter(order::Column::OrderNo.eq(order_no))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_no)))?;

    let mut model: order::ActiveModel = o.into();
    model.status = Set("delivered".to_string());
    model.cards_snapshot = Set(Some(cards_snapshot.to_string()));
    model.pay_time = Set(Some(chrono::Utc::now()));
    model.updated_at = Set(chrono::Utc::now());
    model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn set_post_action_result(
    db: &DatabaseConnection,
    order_no: &str,
    result: &str,
    status: &str,
) -> AppResult<()> {
    let o = order::Entity::find()
        .filter(order::Column::OrderNo.eq(order_no))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_no)))?;

    let mut model: order::ActiveModel = o.into();
    model.post_action_result = Set(Some(result.to_string()));
    model.post_action_status = Set(Some(status.to_string()));
    model.updated_at = Set(chrono::Utc::now());
    model.update(db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn list_orders(
    db: &DatabaseConnection,
    page: u64,
    per_page: u64,
    status_filter: Option<String>,
    email_filter: Option<String>,
) -> AppResult<(Vec<order::Model>, u64)> {
    let mut query = order::Entity::find().order_by_desc(order::Column::CreatedAt);

    if let Some(status) = status_filter {
        query = query.filter(order::Column::Status.eq(status));
    }
    if let Some(email) = email_filter {
        query = query.filter(order::Column::Email.contains(&email));
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

pub async fn get_order(db: &DatabaseConnection, id: i32) -> AppResult<order::Model> {
    order::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", id)))
}

pub fn to_order_response(o: order::Model, hide_cards: bool) -> OrderResponse {
    OrderResponse {
        id: o.id,
        order_no: o.order_no,
        product_id: o.product_id,
        product_name: None,
        variant_id: o.variant_id,
        variant_name: o.variant_name,
        quantity: o.quantity,
        total_amount: o.total_amount,
        email: o.email,
        payment_method: o.payment_method,
        payment_channel: o.payment_channel,
        status: o.status,
        trade_no: o.trade_no,
        pay_time: o.pay_time,
        aff_code: o.aff_code,
        cards_snapshot: if hide_cards { None } else { o.cards_snapshot },
        post_action_result: o.post_action_result,
        post_action_status: o.post_action_status,
        coupon_code: o.coupon_code,
        discount_amount: o.discount_amount,
        query_token: None, // Never leak query_token in general responses
        created_at: o.created_at,
        updated_at: o.updated_at,
    }
}

pub async fn enrich_orders_with_product_names(
    db: &DatabaseConnection,
    orders: Vec<order::Model>,
) -> AppResult<Vec<OrderResponse>> {
    let product_ids: Vec<i32> = orders.iter().map(|o| o.product_id).collect::<std::collections::HashSet<_>>().into_iter().collect();

    let products = product::Entity::find()
        .filter(product::Column::Id.is_in(product_ids))
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let name_map: HashMap<i32, String> = products.into_iter().map(|p| (p.id, p.name)).collect();

    Ok(orders
        .into_iter()
        .map(|o| {
            let product_name = name_map.get(&o.product_id).cloned();
            let mut resp = to_order_response(o, false);
            resp.product_name = product_name;
            resp
        })
        .collect())
}
