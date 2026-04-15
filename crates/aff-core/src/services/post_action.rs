use std::time::Duration;

use aff_common::config::AppConfig;
use aff_common::error::{AppError, AppResult};
use aff_entity::entities::order;
use tracing::{info, warn};

pub async fn execute_post_action(
    action_type: &str,
    action_value: &str,
    order: &order::Model,
    config: Option<&AppConfig>,
) -> AppResult<String> {
    match action_type {
        "webhook" => execute_webhook(action_value, order).await,
        "command" => {
            let allowed = config
                .map(|c| c.security.allow_command_action)
                .unwrap_or(false);
            if !allowed {
                return Err(AppError::Forbidden(
                    "Command execution is disabled. Set security.allow_command_action=true to enable.".to_string()
                ));
            }
            execute_command(action_value, order).await
        }
        _ => Err(AppError::BadRequest(format!(
            "Unknown post_pay_action_type: {}",
            action_type
        ))),
    }
}

async fn execute_webhook(url: &str, order: &order::Model) -> AppResult<String> {
    let payload = serde_json::json!({
        "order_no": order.order_no,
        "product_id": order.product_id,
        "quantity": order.quantity,
        "total_amount": order.total_amount,
        "email": order.email,
        "payment_method": order.payment_method,
        "status": order.status,
        "cards_snapshot": order.cards_snapshot,
    });

    info!(url = %url, order_no = %order.order_no, "Executing post-pay webhook");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(format!("HTTP client error: {}", e)))?;

    let resp = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Webhook request failed: {}", e)))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .unwrap_or_else(|_| "".to_string());

    let result = format!("HTTP {} - {}", status.as_u16(), body);
    if status.is_success() {
        info!(order_no = %order.order_no, "Webhook succeeded: {}", result);
        Ok(result)
    } else {
        warn!(order_no = %order.order_no, "Webhook returned non-success: {}", result);
        Err(AppError::Internal(format!(
            "Webhook returned non-success: {}",
            result
        )))
    }
}

async fn execute_command(command: &str, order: &order::Model) -> AppResult<String> {
    info!(command = %command, order_no = %order.order_no, "Executing post-pay command");

    let child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("ORDER_NO", &order.order_no)
        .env("EMAIL", &order.email)
        .env("AMOUNT", order.total_amount.to_string())
        .env("PRODUCT_ID", order.product_id.to_string())
        .env("QUANTITY", order.quantity.to_string())
        .env("PAYMENT_METHOD", &order.payment_method)
        .env("STATUS", &order.status)
        .env(
            "CARDS_SNAPSHOT",
            order.cards_snapshot.as_deref().unwrap_or(""),
        )
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| AppError::Internal(format!("Command spawn failed: {}", e)))?;

    let timeout = Duration::from_secs(30);

    let wait_result = tokio::time::timeout(timeout, child.wait_with_output()).await;

    match wait_result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);
            let result_str = format!(
                "exit_code={}, stdout={}, stderr={}",
                exit_code,
                stdout.trim(),
                stderr.trim()
            );

            if output.status.success() {
                info!(order_no = %order.order_no, "Command succeeded: {}", result_str);
                Ok(result_str)
            } else {
                warn!(order_no = %order.order_no, "Command failed: {}", result_str);
                Err(AppError::Internal(format!("Command failed: {}", result_str)))
            }
        }
        Ok(Err(e)) => {
            let msg = format!("Command execution error: {}", e);
            warn!(order_no = %order.order_no, "{}", msg);
            Err(AppError::Internal(msg))
        }
        Err(_) => {
            // Timeout — kill_on_drop will handle cleanup
            let msg = "Command timed out after 30 seconds".to_string();
            warn!(order_no = %order.order_no, "{}", msg);
            Err(AppError::Internal(msg))
        }
    }
}
