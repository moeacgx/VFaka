use aff_common::config::AppConfig;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_JWT_SECRET: &str = "change-me-in-production-use-a-long-random-string";

async fn init_database(config: &AppConfig) -> DatabaseConnection {
    let db = Database::connect(&config.database.url)
        .await
        .expect("Failed to connect to database");

    aff_entity::migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed");
    db
}

async fn init_default_admin(db: &DatabaseConnection, config: &AppConfig) {
    use sea_orm::*;
    use aff_entity::entities::admin;

    let existing = admin::Entity::find()
        .filter(admin::Column::Username.eq(&config.admin.username))
        .one(db)
        .await
        .expect("Failed to query admin");

    if existing.is_none() {
        // Generate random password if using default
        let password = if config.admin.password == "admin123" {
            use rand::Rng;
            let random_pass: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            tracing::warn!("==============================================");
            tracing::warn!("  DEFAULT ADMIN PASSWORD DETECTED");
            tracing::warn!("  Generated random password: {}", random_pass);
            tracing::warn!("  Please save this password now!");
            tracing::warn!("==============================================");
            random_pass
        } else {
            config.admin.password.clone()
        };

        let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
            .expect("Failed to hash password");

        let new_admin = admin::ActiveModel {
            username: Set(config.admin.username.clone()),
            password_hash: Set(password_hash),
            role: Set("super_admin".to_string()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        admin::Entity::insert(new_admin)
            .exec(db)
            .await
            .expect("Failed to create default admin");

        tracing::info!("Default admin '{}' created", config.admin.username);
    }
}

fn check_security(config: &AppConfig) {
    if config.jwt.secret == DEFAULT_JWT_SECRET {
        tracing::warn!("==============================================");
        tracing::warn!("  WARNING: Using default JWT secret!");
        tracing::warn!("  This is insecure for production.");
        tracing::warn!("  Set jwt.secret in config.toml or via");
        tracing::warn!("  AFF_JWT_SECRET environment variable.");
        tracing::warn!("==============================================");
    }

    if !config.security.allow_command_action {
        tracing::info!("Post-pay command execution is DISABLED (default)");
    } else {
        tracing::warn!("Post-pay command execution is ENABLED - ensure this is intended");
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,aff=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Loading configuration...");
    let config = AppConfig::load().expect("Failed to load configuration");

    check_security(&config);

    tracing::info!("Initializing database...");
    let db = init_database(&config).await;

    init_default_admin(&db, &config).await;

    let host = config.server.host.clone();
    let port = config.server.port;

    tracing::info!("Starting AFF Card Shop on {}:{}", host, port);

    let db_data = actix_web::web::Data::new(db.clone());
    let config_data = actix_web::web::Data::new(config);

    // Start order timeout cleanup task
    aff_core::tasks::order_timeout::start_cleanup_task(Arc::new(db));

    actix_web::HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        let mut app = actix_web::App::new()
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(db_data.clone())
            .app_data(config_data.clone())
            .configure(aff_api::routes::configure)
            .route("/health", actix_web::web::get().to(|| async {
                actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
            }));

        // Serve uploaded files
        let uploads_path = std::path::Path::new("data/uploads");
        if uploads_path.exists() {
            app = app.service(actix_files::Files::new("/uploads", "data/uploads"));
        }

        // Serve Vue frontend static files if dist/ exists
        let dist_path = std::path::Path::new("frontend/dist");
        if dist_path.exists() {
            app = app
                .service(actix_files::Files::new("/assets", "frontend/dist/assets"))
                .default_service(
                    actix_files::Files::new("/", "frontend/dist")
                        .index_file("index.html")
                        .default_handler(|req: actix_web::dev::ServiceRequest| {
                            let (http_req, _) = req.into_parts();
                            async {
                                let response = actix_files::NamedFile::open_async("frontend/dist/index.html")
                                    .await?
                                    .into_response(&http_req);
                                Ok(actix_web::dev::ServiceResponse::new(http_req, response))
                            }
                        }),
                );
        }

        app
    })
    .bind((host, port))?
    .run()
    .await
}
