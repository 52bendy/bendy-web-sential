mod api;
mod config;
mod db;
mod error;
mod gateway;
mod middleware;
mod security;
mod types;

use std::sync::Arc;
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::gateway::proxy::{AppState, start_gateway};
use crate::error::AppError;
use crate::middleware::ratelimit::RateLimiters;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::retry::RetryClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json())
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "bendy-web-sential starting");

    let db = db::init(&config.database_url)?;

    if !superuser_exists(&db) {
        let hash = bcrypt::hash("bendy2024", bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::ConfigError(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let conn = db.lock().map_err(|_| AppError::InternalError)?;
        conn.execute(
            "INSERT INTO bws_admin_users (username, password_hash, role, active, created_at, updated_at)
             VALUES ('admin', ?1, 'superadmin', 1, ?2, ?2)",
            rusqlite::params![hash, now],
        )?;
        tracing::info!("created default admin user: admin / bendy2024");
    }

    let rate_limiters = RateLimiters::new(&config.rate_limit);
    let cb = Arc::new(CircuitBreaker::new(config.circuit_breaker.clone()));
    let retry_client = Arc::new(RetryClient::new(config.retry.clone()));

    let state = Arc::new(AppState {
        db: db.clone(),
        config: config.clone(),
        rate_limiters,
        circuit_breaker: cb.clone(),
        retry_client,
    });

    let gateway_state = state.clone();
    let _gw = tokio::spawn(async move {
        start_gateway(gateway_state).await;
    });

    let admin = build_admin_server(db.clone(), &config, cb);
    let admin_port = config.admin_port;
    let _admin = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", admin_port)).await.unwrap();
        tracing::info!(port = %admin_port, "admin API listening");
        axum::serve(listener, admin).await.unwrap();
    });

    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");
    Ok(())
}

fn build_admin_server(db: DbPool, config: &AppConfig, cb: Arc<CircuitBreaker>) -> axum::Router {
    use axum::routing::get;

    let auth = api::auth::router(db.clone(), config);
    let gateway_api = api::domains::router(db.clone());
    let audit_api = api::audit::router(db.clone());
    let metrics_api = api::metrics::router(db.clone(), cb);
    let health = Router::new().route("/health", get(|| async { "ok" }));

    axum::Router::new()
        .merge(health)
        .merge(auth)
        .merge(gateway_api)
        .merge(audit_api)
        .merge(metrics_api)
        .layer(TraceLayer::new_for_http())
}

fn superuser_exists(db: &DbPool) -> bool {
    let conn = match db.lock() { Ok(c) => c, Err(_) => return false };
    conn.query_row(
        "SELECT 1 FROM bws_admin_users WHERE role = 'superadmin' AND active = 1 LIMIT 1",
        [],
        |_| Ok(true),
    ).unwrap_or(false)
}