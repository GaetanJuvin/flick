mod app;
mod config;
mod error;
mod auth;
mod middleware;
mod cache;
mod db;
mod evaluation;
mod domains;

use crate::config::AppConfig;
use tracing::info;

#[tokio::main]
async fn main() {
    // Load .env from project root (two levels up from packages/server-rust/)
    let _ = dotenvy::from_filename("../../.env");
    let _ = dotenvy::dotenv();

    let config = AppConfig::from_env();

    // Initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .init();

    // Connect to PostgreSQL
    let db = db::connect(&config.database_url).await;
    info!("Connected to PostgreSQL");

    // Run migrations
    db::migrate::run(&db).await;
    info!("Migrations complete");

    // Connect to Redis
    let redis = cache::connect(&config.redis_url).await;
    info!("Connected to Redis");

    let state = app::AppState {
        db,
        redis,
        config: config.clone(),
    };

    let app = app::build(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("Flick server (Rust) listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received SIGINT, shutting down"),
        _ = terminate => info!("Received SIGTERM, shutting down"),
    }
}
