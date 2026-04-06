use sea_orm::*;
use std::sync::Arc;

use aff_entity::entities::{card, order};

/// Cleanup expired pending orders (15 min timeout).
/// Also recover orders stuck in "processing" for >5 min (rollback to "paid").
/// Releases locked cards back to available for expired orders.
pub async fn cleanup_expired_orders(db: &DatabaseConnection) {
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

        // Mark order as failed
        let mut am: order::ActiveModel = o.clone().into();
        am.status = Set("failed".to_string());
        am.updated_at = Set(chrono::Utc::now());
        let _ = am.update(db).await;

        tracing::info!("Expired order {} cleaned up", o.order_no);
    }

    if !expired.is_empty() {
        // Update product stock counts
        let product_ids: Vec<i32> = expired.iter().map(|o| o.product_id).collect();
        for pid in product_ids {
            let count = card::Entity::find()
                .filter(card::Column::ProductId.eq(pid))
                .filter(card::Column::Status.eq("available"))
                .count(db)
                .await
                .unwrap_or(0);

            use aff_entity::entities::product;
            if let Ok(Some(p)) = product::Entity::find_by_id(pid).one(db).await {
                let mut am: product::ActiveModel = p.into();
                am.stock_count = Set(count as i32);
                let _ = am.update(db).await;
            }
        }

        tracing::info!("Cleaned up {} expired orders", expired.len());
    }
}

/// Start the background cleanup loop (runs every 60 seconds)
pub fn start_cleanup_task(db: Arc<DatabaseConnection>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_expired_orders(&db).await;
        }
    });
}
