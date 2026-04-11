use sea_orm::*;
use std::collections::HashSet;
use std::sync::Arc;

use aff_common::config::AppConfig;
use aff_entity::entities::{card, order, product, product_variant};

/// Cleanup expired pending orders (15 min timeout).
/// Also recover orders stuck in "processing" for >5 min (rollback to "paid").
/// Releases locked cards back to available for expired orders.
/// Retries failed post-pay actions (max 3 attempts).
pub async fn cleanup_expired_orders(db: &DatabaseConnection, config: &AppConfig) {
    let pending_cutoff = chrono::Utc::now() - chrono::Duration::minutes(15);
    let processing_cutoff = chrono::Utc::now() - chrono::Duration::minutes(5);

    // Recover stuck "processing" orders back to "paid"
    let stuck = order::Entity::find()
        .filter(order::Column::Status.eq("processing"))
        .filter(order::Column::UpdatedAt.lt(processing_cutoff))
        .all(db)
        .await;

    if let Ok(stuck_orders) = stuck {
        for o in &stuck_orders {
            let mut am: order::ActiveModel = o.clone().into();
            am.status = Set("paid".to_string());
            am.updated_at = Set(chrono::Utc::now());
            let _ = am.update(db).await;
            tracing::warn!("Recovered stuck processing order {} back to paid", o.order_no);
        }
    }

    let expired = order::Entity::find()
        .filter(order::Column::Status.eq("pending"))
        .filter(order::Column::CreatedAt.lt(pending_cutoff))
        .all(db)
        .await;

    let expired = match expired {
        Ok(orders) => orders,
        Err(e) => {
            tracing::error!("Failed to query expired orders: {}", e);
            return;
        }
    };

    for o in &expired {
        // Release locked cards for this order
        let locked_cards = card::Entity::find()
            .filter(card::Column::OrderId.eq(Some(o.id)))
            .filter(card::Column::Status.eq("locked"))
            .all(db)
            .await;

        if let Ok(cards) = locked_cards {
            for c in cards {
                let mut am: card::ActiveModel = c.into();
                am.status = Set("available".to_string());
                am.order_id = Set(None);
                let _ = am.update(db).await;
            }
        }

        // Mark order as expired
        let mut am: order::ActiveModel = o.clone().into();
        am.status = Set("expired".to_string());
        am.updated_at = Set(chrono::Utc::now());
        let _ = am.update(db).await;

        tracing::info!("Expired order {} cleaned up", o.order_no);
    }

    if !expired.is_empty() {
        // Update product and variant stock counts (dedup IDs)
        let product_ids: HashSet<i32> = expired.iter().map(|o| o.product_id).collect();
        let variant_ids: HashSet<i32> = expired.iter().filter_map(|o| o.variant_id).collect();

        for pid in product_ids {
            let count = card::Entity::find()
                .filter(card::Column::ProductId.eq(pid))
                .filter(card::Column::Status.eq("available"))
                .count(db)
                .await
                .unwrap_or(0);

            if let Ok(Some(p)) = product::Entity::find_by_id(pid).one(db).await {
                let mut am: product::ActiveModel = p.into();
                am.stock_count = Set(count as i32);
                let _ = am.update(db).await;
            }
        }

        for vid in variant_ids {
            let count = card::Entity::find()
                .filter(card::Column::VariantId.eq(vid))
                .filter(card::Column::Status.eq("available"))
                .count(db)
                .await
                .unwrap_or(0);

            if let Ok(Some(v)) = product_variant::Entity::find_by_id(vid).one(db).await {
                let mut am: product_variant::ActiveModel = v.into();
                am.stock_count = Set(count as i32);
                let _ = am.update(db).await;
            }
        }

        tracing::info!("Cleaned up {} expired orders", expired.len());
    }

    // Retry failed post-pay actions (max 3 attempts)
    retry_failed_post_actions(db, config).await;
}

/// Retry orders with post_action_status = "failed", up to 3 attempts.
async fn retry_failed_post_actions(db: &DatabaseConnection, config: &AppConfig) {
    let failed_orders = order::Entity::find()
        .filter(order::Column::Status.eq("delivered"))
        .filter(order::Column::PostActionStatus.eq("failed"))
        .all(db)
        .await;

    let failed_orders = match failed_orders {
        Ok(orders) => orders,
        Err(_) => return,
    };

    for o in &failed_orders {
        // Count retries from post_action_result prefix
        let retry_count = o
            .post_action_result
            .as_deref()
            .unwrap_or("")
            .matches("[RETRY")
            .count();

        if retry_count >= 3 {
            continue; // Max retries reached
        }

        let product_model = product::Entity::find_by_id(o.product_id)
            .one(db)
            .await
            .ok()
            .flatten();

        if let Some(product_model) = product_model {
            if let (Some(action_type), Some(action_value)) = (
                &product_model.post_pay_action_type,
                &product_model.post_pay_action_value,
            ) {
                if !action_type.is_empty() && !action_value.is_empty() {
                    tracing::info!(
                        order_no = %o.order_no,
                        retry = retry_count + 1,
                        "Retrying failed post-pay action"
                    );

                    match crate::services::post_action::execute_post_action(
                        action_type,
                        action_value,
                        o,
                        Some(config),
                    )
                    .await
                    {
                        Ok(result) => {
                            let mut am: order::ActiveModel = o.clone().into();
                            am.post_action_result =
                                Set(Some(format!("[RETRY {}] {}", retry_count + 1, result)));
                            am.post_action_status = Set(Some("success".to_string()));
                            am.updated_at = Set(chrono::Utc::now());
                            let _ = am.update(db).await;
                            tracing::info!(
                                order_no = %o.order_no,
                                "Post-pay action retry succeeded"
                            );
                        }
                        Err(e) => {
                            let mut am: order::ActiveModel = o.clone().into();
                            am.post_action_result = Set(Some(format!(
                                "[RETRY {}] ERROR: {}",
                                retry_count + 1,
                                e
                            )));
                            am.updated_at = Set(chrono::Utc::now());
                            let _ = am.update(db).await;
                            tracing::warn!(
                                order_no = %o.order_no,
                                "Post-pay action retry {} failed: {}",
                                retry_count + 1,
                                e
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Start the background cleanup loop (runs every 60 seconds)
pub fn start_cleanup_task(db: Arc<DatabaseConnection>, config: Arc<AppConfig>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_expired_orders(&db, &config).await;
        }
    });
}
