use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tracing::info;

use aff_common::config::AppConfig;
use aff_common::error::{AppError, AppResult};
use aff_common::id_gen::generate_order_no;
use aff_core::services::{
    aff_service, card_service, coupon_service, order_service, payment_config_service,
    product_service,
};
use aff_entity::dto::CreateOrderDto;
use aff_payment::create_provider;
use aff_payment::provider::PaymentRequest;

fn get_client_ip(req: &HttpRequest) -> String {
    req.connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string()
}

fn determine_channel(method: &str) -> AppResult<&'static str> {
    match method {
        "alipay" | "wxpay" | "qqpay" => Ok("epay"),
        "usdt_trc20" | "trx" | "usdt_erc20" | "usdc_erc20" => Ok("tokenpay"),
        _ => Err(AppError::BadRequest(format!(
            "Unknown payment method: {}",
            method
        ))),
    }
}

fn is_method_allowed(product: &aff_entity::dto::ProductResponse, method: &str) -> bool {
    match method {
        "alipay" => product.allow_alipay,
        "wxpay" => product.allow_wxpay,
        "qqpay" => product.allow_qqpay,
        "usdt_trc20" => product.allow_usdt_trc20,
        "trx" => product.allow_trx,
        "usdt_erc20" => product.allow_usdt_erc20,
        _ => false,
    }
}

pub async fn create_order(
    db: web::Data<DatabaseConnection>,
    config: web::Data<AppConfig>,
    req: HttpRequest,
    body: web::Json<CreateOrderDto>,
) -> AppResult<HttpResponse> {
    let dto = body.into_inner();
    let client_ip = get_client_ip(&req);

    // 1. Validate product
    let product = product_service::get_product(db.get_ref(), dto.product_id).await?;
    if !product.is_active {
        return Err(AppError::BadRequest("Product is not available".into()));
    }

    // 2. Resolve variant (if applicable)
    let has_variants = !product.variants.is_empty();
    let (unit_price, variant_id, variant_name, check_stock) = if has_variants {
        let vid = dto.variant_id.ok_or_else(|| {
            AppError::BadRequest("This product requires a variant selection".into())
        })?;
        let variant = product
            .variants
            .iter()
            .find(|v| v.id == vid && v.is_active)
            .ok_or_else(|| AppError::BadRequest("Invalid or inactive variant".into()))?;
        (variant.price, Some(vid), Some(variant.name.clone()), variant.stock_count)
    } else {
        (product.price, None, None, product.stock_count)
    };

    // 3. Check stock
    if check_stock < dto.quantity {
        return Err(AppError::Conflict(format!(
            "Insufficient stock: available {}, requested {}",
            check_stock, dto.quantity
        )));
    }

    // 4. Quantity bounds
    if dto.quantity < 1 {
        return Err(AppError::BadRequest("Quantity must be at least 1".into()));
    }
    if dto.quantity < product.min_quantity || dto.quantity > product.max_quantity {
        return Err(AppError::BadRequest(format!(
            "Quantity must be between {} and {}",
            product.min_quantity, product.max_quantity
        )));
    }

    // 4. Check payment method allowed
    if !is_method_allowed(&product, &dto.payment_method) {
        return Err(AppError::BadRequest(format!(
            "Payment method '{}' is not allowed for this product",
            dto.payment_method
        )));
    }

    // 5. Generate order_no
    let order_no = generate_order_no();

    // 6. Determine payment channel
    let channel = determine_channel(&dto.payment_method)?;

    // 7. Calculate total and apply coupon
    let subtotal = unit_price * dto.quantity as f64;
    let mut discount_amount = 0.0;
    let mut coupon_code_used: Option<String> = None;

    if let Some(ref code) = dto.coupon_code {
        let code = code.trim().to_uppercase();
        if !code.is_empty() {
            let result =
                coupon_service::validate_coupon(db.get_ref(), &code, dto.product_id, subtotal)
                    .await?;
            if !result.valid {
                return Err(AppError::BadRequest(
                    result.message.unwrap_or_else(|| "Invalid coupon".to_string()),
                ));
            }
            discount_amount = result.discount_amount.unwrap_or(0.0);
            coupon_code_used = Some(code);
        }
    }

    let total_amount = (subtotal - discount_amount).max(0.01);

    // Enforce minimum order amount to prevent near-free purchases
    if total_amount < 0.10 {
        return Err(AppError::BadRequest(
            "Order amount too low after discount (minimum ¥0.10)".into(),
        ));
    }

    // 8. Look up aff_user email if aff_code provided
    let aff_user_email = if let Some(ref code) = dto.aff_code {
        aff_service::get_user_by_code(db.get_ref(), code)
            .await?
            .map(|u| u.email)
    } else {
        None
    };

    // 9. Create order record
    let order = order_service::create_order(
        db.get_ref(),
        order_no.clone(),
        dto.product_id,
        dto.quantity,
        total_amount,
        dto.email.clone(),
        dto.payment_method.clone(),
        channel.to_string(),
        dto.aff_code.clone(),
        aff_user_email,
        Some(client_ip.clone()),
        coupon_code_used.clone(),
        discount_amount,
        variant_id,
        variant_name,
    )
    .await?;

    // 9b. Atomically increment coupon usage (CAS — will fail if limit reached by concurrent request)
    if let Some(ref code) = coupon_code_used {
        coupon_service::use_coupon(db.get_ref(), code).await?;
    }

    // 10. Lock cards (bind to order)
    let _locked_cards = card_service::lock_cards(db.get_ref(), dto.product_id, variant_id, dto.quantity, order.id).await?;

    // 11. Load payment config from DB
    let configs = payment_config_service::list_configs(db.get_ref()).await?;
    let pay_config = configs
        .iter()
        .find(|c| c.channel == channel && c.is_active)
        .ok_or_else(|| {
            AppError::BadRequest(format!("Payment channel '{}' is not configured", channel))
        })?;

    // 12. Create payment provider and submit
    let provider = create_provider(channel, &pay_config.config_json)?;

    let base_url = config.get_public_base_url();

    let notify_url = match channel {
        "epay" => format!("{}/api/v1/pay/epay/notify", base_url),
        "tokenpay" => format!("{}/api/v1/pay/tokenpay/notify", base_url),
        _ => format!("{}/api/v1/pay/{}/notify", base_url, channel),
    };

    let return_url = format!("{}/api/v1/pay/epay/return?order_no={}", base_url, order_no);

    let payment_method_enum: aff_common::types::PaymentMethod =
        serde_json::from_value(serde_json::Value::String(dto.payment_method.clone()))
            .map_err(|_| {
                AppError::BadRequest(format!(
                    "Invalid payment method: {}",
                    dto.payment_method
                ))
            })?;

    let pay_req = PaymentRequest {
        order_no: order_no.clone(),
        product_name: product.name.clone(),
        amount: total_amount,
        payment_method: payment_method_enum,
        user_email: dto.email.clone(),
        client_ip,
        notify_url,
        return_url,
    };

    let pay_resp = provider.create_payment(pay_req).await?;

    // Save trade_no to order
    if !pay_resp.trade_no.is_empty() {
        order_service::update_order_trade_no(db.get_ref(), &order_no, &pay_resp.trade_no).await?;
    }

    info!(
        order_no = %order_no,
        order_id = order.id,
        "Order created successfully"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "order_no": order_no,
        "payment_url": pay_resp.pay_url,
        "qr_code": pay_resp.qr_code,
        "query_token": order.query_token,
    })))
}

#[derive(Debug, Deserialize)]
pub struct OrderTokenQuery {
    pub token: Option<String>,
    pub email: Option<String>,
}

pub async fn get_order_status(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    query: web::Query<OrderTokenQuery>,
) -> AppResult<HttpResponse> {
    let order_no = path.into_inner();

    let order = order_service::get_order_by_no(db.get_ref(), &order_no)
        .await?
        .ok_or_else(|| AppError::NotFound("Order not found".into()))?;

    // Token-based auth (primary for new orders)
    if let Some(ref token) = query.token {
        if let Some(ref order_token) = order.query_token {
            if token == order_token {
                let hide_cards = order.status != "delivered";
                let resp = order_service::to_order_response(order, hide_cards);
                return Ok(HttpResponse::Ok().json(resp));
            }
        }
        return Err(AppError::NotFound("Order not found".into()));
    }

    // Email fallback (legacy: only for old orders without query_token)
    if let Some(ref email) = query.email {
        if order.query_token.is_some() {
            // New order with token — don't allow email-only access
            return Err(AppError::BadRequest("Token required for this order".into()));
        }
        if &order.email != email {
            return Err(AppError::NotFound("Order not found".into()));
        }
        let hide_cards = order.status != "delivered";
        let resp = order_service::to_order_response(order, hide_cards);
        return Ok(HttpResponse::Ok().json(resp));
    }

    Err(AppError::BadRequest("Token or email required".into()))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/orders", web::post().to(create_order))
        .route("/orders/{order_no}", web::get().to(get_order_status));
}
