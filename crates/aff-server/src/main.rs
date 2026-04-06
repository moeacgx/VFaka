use aff_common::config::AppConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,aff=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Loading configuration...");
    let config = AppConfig::load().expect("Failed to load configuration");

    tracing::info!(
        "Starting AFF Card Shop server on {}:{}",
        config.server.host,
        config.server.port
    );

    // TODO: Initialize database connection
    // TODO: Run migrations
    // TODO: Initialize default admin

    let host = config.server.host.clone();
    let port = config.server.port;

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .configure(aff_api::routes::configure)
    })
    .bind((host, port))?
    .run()
    .await
}
