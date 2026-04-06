use serde::Serialize;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
    pub enabled: bool,
}

pub async fn send_message(config: &TelegramConfig, text: &str) -> Result<(), String> {
    if !config.enabled || config.bot_token.is_empty() || config.chat_id.is_empty() {
        return Ok(());
    }

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );

    let payload = serde_json::json!({
        "chat_id": config.chat_id,
        "text": text,
        "parse_mode": "HTML",
        "disable_web_page_preview": true,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Telegram request failed: {}", e))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Telegram API error: {}", body));
    }

    Ok(())
}

pub fn send_payment_notification(
    config: TelegramConfig,
    order_no: String,
    email: String,
    amount: f64,
    product_name: String,
    quantity: i32,
) {
    tokio::spawn(async move {
        let text = format!(
            "<b>Payment Received</b>\n\nOrder: <code>{}</code>\nProduct: {}\nQuantity: {}\nAmount: {:.2}\nEmail: {}",
            order_no, product_name, quantity, amount, email
        );
        if let Err(e) = send_message(&config, &text).await {
            error!("Failed to send Telegram payment notification: {}", e);
        } else {
            info!(order_no = %order_no, "Telegram payment notification sent");
        }
    });
}

pub fn send_withdrawal_notification(
    config: TelegramConfig,
    email: String,
    amount: f64,
    currency: String,
    chain: String,
    wallet: String,
) {
    tokio::spawn(async move {
        let text = format!(
            "<b>Withdrawal Request</b>\n\nEmail: {}\nAmount: {:.2} {}\nChain: {}\nWallet: <code>{}</code>",
            email, amount, currency, chain, wallet
        );
        if let Err(e) = send_message(&config, &text).await {
            error!("Failed to send Telegram withdrawal notification: {}", e);
        } else {
            info!(email = %email, "Telegram withdrawal notification sent");
        }
    });
}
