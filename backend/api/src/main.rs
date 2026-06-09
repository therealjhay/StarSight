use std::sync::Arc;

use axum::Router;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod config;
mod db;
mod indexer;
mod models;
mod routes;

use config::Config;

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    /// Broadcast channel for real-time prediction events pushed to WebSocket clients.
    pub tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    // Load .env file from the workspace root (two levels up from binary source).
    dotenv::dotenv().ok();

    // Initialize structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "starsight_api=info,tower_http=info".into()),
        )
        .init();

    // Parse typed configuration from environment variables.
    let config = Config::from_env();

    // Set up the PostgreSQL connection pool and run inline migrations.
    let pool = db::create_pool(&config.database_url).await;
    db::run_migrations(&pool).await;

    // Create a broadcast channel for WebSocket prediction events (capacity: 256).
    let (tx, _rx) = broadcast::channel::<String>(256);

    let state = AppState {
        pool: pool.clone(),
        tx: tx.clone(),
    };

    // Spawn the background Soroban event indexer.
    let indexer_state = state.clone();
    let indexer_config = config.clone();
    tokio::spawn(async move {
        indexer::run(indexer_state, indexer_config).await;
    });

    // Build the Axum router with all route modules and middleware.
    let app = Router::new()
        .merge(routes::router())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    let addr = format!("0.0.0.0:{}", config.api_port);
    tracing::info!("StarSight API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
