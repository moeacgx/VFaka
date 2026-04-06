use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub enabled: bool,
}

async fn send_email(config: &SmtpConfig, to: &str, subject: &str, body: &str) -> Result<(), String> {
    if !config.enabled || config.host.is_empty() {
        return Ok(());
    }

    let email = Message::builder()
        .from(
            config
                .from_address
                .parse()
                .map_err(|e| format!("Invalid from address: {}", e))?,
        )
        .to(to.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let mailer = if config.port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .credentials(creds)
            .port(config.port)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
            .map_err(|e| format!("SMTP STARTTLS error: {}", e))?
            .credentials(creds)
            .port(config.port)
            .build()
    };

    mailer
        .send(email)
        .await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}

pub fn send_order_confirmation(
    config: SmtpConfig,
    to_email: String,
    order_no: String,
    product_name: String,
    quantity: i32,
    amount: f64,
    cards: String,
) {
    tokio::spawn(async move {
        let subject = format!("Order Confirmation - {}", order_no);
        let body = format!(
            "Your order has been processed successfully.\n\n\
             Order No: {}\n\
             Product: {}\n\
             Quantity: {}\n\
             Amount: {:.2}\n\n\
             --- Card Details ---\n\
             {}\n\n\
             Thank you for your purchase.",
            order_no, product_name, quantity, amount, cards
        );

        if let Err(e) = send_email(&config, &to_email, &subject, &body).await {
            error!(order_no = %order_no, "Failed to send order confirmation email: {}", e);
        } else {
            info!(order_no = %order_no, to = %to_email, "Order confirmation email sent");
        }
    });
}

pub fn send_withdrawal_status(
    config: SmtpConfig,
    to_email: String,
    amount: f64,
    currency: String,
    status: String,
    tx_hash: Option<String>,
    note: Option<String>,
) {
    tokio::spawn(async move {
        let subject = format!("Withdrawal {} - {:.2} {}", status, amount, currency);
        let mut body = format!(
            "Your withdrawal request has been {}.\n\n\
             Amount: {:.2} {}\n\
             Status: {}",
            status, amount, currency, status
        );
        if let Some(hash) = tx_hash {
            body.push_str(&format!("\nTransaction Hash: {}", hash));
        }
        if let Some(n) = note {
            body.push_str(&format!("\nNote: {}", n));
        }

        if let Err(e) = send_email(&config, &to_email, &subject, &body).await {
            error!("Failed to send withdrawal status email: {}", e);
        } else {
            info!(to = %to_email, "Withdrawal status email sent");
        }
    });
}

pub async fn send_test_email(config: &SmtpConfig, to: &str) -> Result<(), String> {
    send_email(
        config,
        to,
        "AFF Card Shop - Test Email",
        "This is a test email from AFF Card Shop. If you received this, SMTP is configured correctly.",
    )
    .await
}
