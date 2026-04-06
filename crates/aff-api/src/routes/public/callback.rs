use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait};
use tracing::{error, info, warn};

use aff_common::config::AppConfig;
use aff_common::error::{AppError, AppResult};
use aff_core::services::{
    aff_service, card_service, order_service, payment_config_service, post_action, product_service,
    settings_service,
};
use aff_entity::entities::card;
use aff_payment::create_provider;
use aff_payment::provider::CallbackRawData;

async fn load_telegram_config(db: &DatabaseConnection) -> aff_notify::telegram::TelegramConfig {
    let bot_token = settings_service::get_setting(db, "telegram_bot_token")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let chat_id = settings_service::get_setting(db, "telegram_chat_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let enabled = settings_service::get_setting(db, "telegram_enabled")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "false".to_string())
        == "true";

    aff_notify::telegram::TelegramConfig {
        bot_token,
        chat_id,
        enabled,
    }
}

async fn load_smtp_config(db: &DatabaseConnection) -> aff_notify::email::SmtpConfig {
    let host = settings_service::get_setting(db, "smtp_host").await.ok().flatten().unwrap_or_default();
    let port: u16 = settings_service::get_setting(db, "smtp_port").await.ok().flatten()
        .unwrap_or_else(|| "465".to_string()).parse().unwrap_or(465);
    let username = settings_service::get_setting(db, "smtp_username").await.ok().flatten().unwrap_or_default();
    let password = settings_service::get_setting(db, "smtp_password").await.ok().flatten().unwrap_or_default();
    let from_address = settings_service::get_setting(db, "smtp_from").await.ok().flatten().unwrap_or_default();
    let enabled = settings_service::get_setting(db, "smtp_enabled").await.ok().flatten()
        .unwrap_or_else(|| "false".to_string()) == "true";

    aff_notify::email::SmtpConfig { host, port, username, password, from_address, enabled }
}

async fn process_paid_order(
    db: &sea_orm::DatabaseConnection,
    order_no: &str,
) -> AppResult<()> {
    // Re-fetch order after status update
    let order = order_service::get_order_by_no(db, order_no)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_no)))?;

    // Deliver cards: find locked cards for this product
    let locked_cards = card_service::list_cards(db, Some(order.product_id), Some("locked".to_string())).await?;
    let card_ids: Vec<i32> = locked_cards
        .iter()
        .take(order.quantity as usize)
        .map(|c| c.id)
        .collect();

    if card_ids.len() < order.quantity as usize {
        warn!(
            order_no = %order_no,
            "Not enough locked cards for delivery: need {}, have {}",
            order.quantity,
            card_ids.len()
        );
    }

    // Deliver cards
    card_service::deliver_cards(db, &card_ids, order.id).await?;

    // Build cards snapshot
    let delivered_cards: Vec<&card::Model> = locked_cards
        .iter()
        .filter(|c| card_ids.contains(&c.id))
        .collect();
    let snapshot: Vec<String> = delivered_cards.iter().map(|c| c.content.clone()).collect();
    let cards_snapshot = snapshot.join("\n");

    // Update order to delivered with cards snapshot
    order_service::deliver_order(db, order_no, &cards_snapshot).await?;

    // Update product sales_count
    let product = product_service::get_product(db, order.product_id).await?;
    let _ = product; // stock_count already updated by card_service

    // Execute post_pay_action if configured
    let order = order_service::get_order_by_no(db, order_no)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_no)))?;

    let product_model = aff_entity::entities::product::Entity::find_by_id(order.product_id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(product_model) = product_model {
        if let (Some(action_type), Some(action_value)) = (
            &product_model.post_pay_action_type,
            &product_model.post_pay_action_value,
        ) {
            if !action_type.is_empty() && !action_value.is_empty() {
                match post_action::execute_post_action(action_type, action_value, &order).await {
                    Ok(result) => {
                        let _ = order_service::set_post_action_result(db, order_no, &result).await;
                    }
                    Err(e) => {
                        error!(order_no = %order_no, "Post-pay action failed: {}", e);
                        let _ = order_service::set_post_action_result(
                            db,
                            order_no,
                            &format!("ERROR: {}", e),
                        )
                        .await;
                    }
                }
            }
        }

        // Update sales_count
        let new_sales = product_model.sales_count + order.quantity;
        use sea_orm::*;
        let mut prod_am: aff_entity::entities::product::ActiveModel = product_model.into();
        prod_am.sales_count = Set(new_sales);
        prod_am.updated_at = Set(chrono::Utc::now());
        prod_am
            .update(db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // Process AFF commission
    let order = order_service::get_order_by_no(db, order_no)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_no)))?;
    if let Err(e) = aff_service::process_commission(db, &order).await {
        error!(order_no = %order_no, "AFF commission processing failed: {}", e);
    }

    // Send notifications
    let tg_config = load_telegram_config(db).await;
    let smtp_config = load_smtp_config(db).await;

    let product_name = aff_entity::entities::product::Entity::find_by_id(order.product_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Product #{}", order.product_id));

    aff_notify::telegram::send_payment_notification(
        tg_config,
        order.order_no.clone(),
        order.email.clone(),
        order.total_amount,
        product_name.clone(),
        order.quantity,
    );

    let cards_for_email = order.cards_snapshot.clone().unwrap_or_default();
    aff_notify::email::send_order_confirmation(
        smtp_config,
        order.email.clone(),
        order.order_no.clone(),
        product_name,
        order.quantity,
        order.total_amount,
        cards_for_email,
    );

    info!(order_no = %order_no, "Order fully processed (paid -> delivered)");
    Ok(())
}

pub async fn epay_notify(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let query_string = req.query_string().to_string();
    info!(qs = %query_string, "Epay notify callback received");

    // Load epay config
    let configs = payment_config_service::list_configs(db.get_ref()).await?;
    let pay_config = configs
        .iter()
        .find(|c| c.channel == "epay" && c.is_active)
        .ok_or_else(|| AppError::Internal("Epay config not found".into()))?;

    let provider = create_provider("epay", &pay_config.config_json)?;

    let raw = CallbackRawData {
        query_string: Some(query_string),
        body: None,
        headers: std::collections::HashMap::new(),
    };

    let cb_data = provider.verify_callback(&raw).await?;

    if !cb_data.is_success {
        info!(order_no = %cb_data.order_no, "Epay callback: trade not successful");
        return Ok(HttpResponse::Ok().body("success"));
    }

    // Find order
    let order = order_service::get_order_by_no(db.get_ref(), &cb_data.order_no)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Order {} not found", cb_data.order_no))
        })?;

    // Skip if already processed
    if order.status != "pending" {
        info!(order_no = %cb_data.order_no, status = %order.status, "Order already processed, skipping");
        return Ok(HttpResponse::Ok().body("success"));
    }

    // Update to paid
    order_service::update_order_status(db.get_ref(), &cb_data.order_no, "paid").await?;
    if !cb_data.trade_no.is_empty() {
        order_service::update_order_trade_no(db.get_ref(), &cb_data.order_no, &cb_data.trade_no)
            .await?;
    }

    // Process paid order (deliver cards, post action, AFF)
    if let Err(e) = process_paid_order(db.get_ref(), &cb_data.order_no).await {
        error!(order_no = %cb_data.order_no, "Failed to process paid order: {}", e);
    }

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("success"))
}

pub async fn tokenpay_notify(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Bytes,
) -> AppResult<HttpResponse> {
    let body_str = String::from_utf8_lossy(&body).to_string();
    info!(body = %body_str, "TokenPay notify callback received");

    // Load tokenpay config
    let configs = payment_config_service::list_configs(db.get_ref()).await?;
    let pay_config = configs
        .iter()
        .find(|c| c.channel == "tokenpay" && c.is_active)
        .ok_or_else(|| AppError::Internal("TokenPay config not found".into()))?;

    let provider = create_provider("tokenpay", &pay_config.config_json)?;

    let mut headers = std::collections::HashMap::new();
    for (key, value) in req.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(key.to_string(), v.to_string());
        }
    }

    let raw = CallbackRawData {
        query_string: None,
        body: Some(body_str),
        headers,
    };

    let cb_data = provider.verify_callback(&raw).await?;

    if !cb_data.is_success {
        info!(order_no = %cb_data.order_no, "TokenPay callback: trade not successful");
        return Ok(HttpResponse::Ok().body("ok"));
    }

    // Find order
    let order = order_service::get_order_by_no(db.get_ref(), &cb_data.order_no)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Order {} not found", cb_data.order_no))
        })?;

    if order.status != "pending" {
        info!(order_no = %cb_data.order_no, status = %order.status, "Order already processed, skipping");
        return Ok(HttpResponse::Ok().body("ok"));
    }

    // Update to paid
    order_service::update_order_status(db.get_ref(), &cb_data.order_no, "paid").await?;
    if !cb_data.trade_no.is_empty() {
        order_service::update_order_trade_no(db.get_ref(), &cb_data.order_no, &cb_data.trade_no)
            .await?;
    }

    // Process paid order
    if let Err(e) = process_paid_order(db.get_ref(), &cb_data.order_no).await {
        error!(order_no = %cb_data.order_no, "Failed to process paid order: {}", e);
    }

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("ok"))
}

pub async fn epay_return(
    req: HttpRequest,
    config: web::Data<AppConfig>,
) -> HttpResponse {
    let order_no = req
        .uri()
        .query()
        .and_then(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "order_no")
                .map(|(_, v)| v.to_string())
        })
        .unwrap_or_default();

    let redirect_url = format!(
        "http://{}:{}/order/{}",
        config.server.host, config.server.port, order_no
    );

    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/pay")
            .route("/epay/notify", web::get().to(epay_notify))
            .route("/epay/return", web::get().to(epay_return))
            .route("/tokenpay/notify", web::post().to(tokenpay_notify)),
    );
}
