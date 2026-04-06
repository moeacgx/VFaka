use aff_common::config::AppConfig;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        .filter(admin::Column::Username.eq(&config.default_admin.username))
        .one(db)
        .await
        .expect("Failed to query admin");

    if existing.is_none() {
        let password_hash = bcrypt::hash(&config.default_admin.password, bcrypt::DEFAULT_COST)
            .expect("Failed to hash password");

        let new_admin = admin::ActiveModel {
            username: Set(config.default_admin.username.clone()),
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

        tracing::info!("Default admin '{}' created", config.default_admin.username);
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

    tracing::info!("Initializing database...");
    let db = init_database(&config).await;

    init_default_admin(&db, &config).await;

    let host = config.server.host.clone();
    let port = config.server.port;

    tracing::info!("Starting AFF Card Shop on {}:{}", host, port);

    let db_data = actix_web::web::Data::new(db);
    let config_data = actix_web::web::Data::new(config);

    actix_web::HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        actix_web::App::new()
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(db_data.clone())
            .app_data(config_data.clone())
            .configure(aff_api::routes::configure)
            .route("/health", actix_web::web::get().to(|| async {
                actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
            }))
    })
    .bind((host, port))?
    .run()
    .await
}
