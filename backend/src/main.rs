mod api;
mod config;
mod db;
mod error;
mod models;
mod radius;
mod utils;

use axum::{
    extract::DefaultBodyLimit,
    http::HeaderValue,
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize environment variables
    dotenv::dotenv().ok();

    // Setup logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,wifi_billing_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();

    tracing::info!("Starting WiFi Hotspot Billing System");

    // Load config
    let config = config::Config::from_env()?;
    tracing::info!("Config loaded: {:?}", config);

    // Database setup
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations applied");

    // Spawn RADIUS server
    let pool_radius = pool.clone();
    let config_radius = config.clone();
    tokio::spawn(async move {
        if let Err(e) = radius::server::start(pool_radius, config_radius).await {
            tracing::error!("RADIUS server error: {}", e);
        }
    });

    // Build router
    let cors = CorsLayer::permissive();

    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Auth routes
        .route("/api/auth/signup", post(api::auth::signup))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/logout", post(api::auth::logout))
        .route(
            "/api/auth/verify-otp",
            post(api::auth::verify_otp),
        )
        // User routes
        .route("/api/user/profile", get(api::user::get_profile))
        .route("/api/user/devices", get(api::user::list_devices))
        .route("/api/user/phones", get(api::user::list_phones))
        .route("/api/user/add-phone", post(api::user::add_phone))
        .route("/api/user/remove-phone", post(api::user::remove_phone))
        .route("/api/user/set-default-phone", post(api::user::set_default_phone))
        // Package routes
        .route("/api/packages", get(api::package::list_packages))
        .route("/api/packages/:id", get(api::package::get_package))
        // Payment routes
        .route("/api/payment/initiate", post(api::payment::initiate_payment))
        .route("/api/payment/status/:transaction_id", get(api::payment::payment_status))
        .route("/api/payment/callback", post(api::payment::mpesa_callback))
        // Quota routes
        .route("/api/quota/status", get(api::quota::quota_status))
        // Admin routes
        .route("/api/admin/login", post(api::admin::login))
        .route("/api/admin/packages", post(api::admin::create_package))
        .route("/api/admin/packages/:id", axum::routing::put(api::admin::update_package))
        .route("/api/admin/packages/:id", axum::routing::delete(api::admin::delete_package))
        .route("/api/admin/users", get(api::admin::list_users))
        .route("/api/admin/transactions", get(api::admin::list_transactions))
        .route("/api/admin/stats", get(api::admin::get_stats))
        .layer(middleware::Next::layer())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(Arc::new(AppState {
            pool,
            config: config.clone(),
        }));

    // Start server
    let addr = format!("{}:{}", config.axum_host, config.axum_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: config::Config,
}

mod handlers {
    use axum::Json;
    use serde_json::json;

    pub async fn health() -> Json<serde_json::Value> {
        Json(json!({ "status": "ok" }))
    }
}
