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
use tower_http::{trace::TraceLayer, services::ServeDir};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::gateway::proxy::{AppState, start_gateway};
use crate::gateway::cache::Caches;
use crate::error::AppError;
use crate::middleware::ratelimit::RateLimiters;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::retry::RetryClient;
use crate::security::{JwtServiceClone, TokenBlacklist};
use crate::api::prometheus::{PrometheusMetrics, router as prometheus_router};

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
    let blacklist = Arc::new(TokenBlacklist::new());
    let metrics = PrometheusMetrics::new();
    let jwt = JwtServiceClone::new(config.jwt_secret.clone(), config.jwt_expiry_secs);
    let caches = Caches::new();

    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        rate_limiters,
        circuit_breaker: cb.clone(),
        retry_client,
        jwt: jwt.clone(),
        caches: caches.clone(),
    };

    let gateway_state = state.clone();
    let _gw = tokio::spawn(async move {
        start_gateway(gateway_state).await;
    });

    let db_for_health = db.clone();
    let _health = tokio::spawn(async move {
        use crate::gateway::health_check::start_health_checker;
        start_health_checker(db_for_health, 30).await;
    });

    let admin = build_admin_server(db.clone(), &config, cb, blacklist, metrics);
    let admin_port = config.admin_port;
    let _admin = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", admin_port)).await.unwrap();
        let incoming = hyper::server::conn::AddrIncoming::from_listener(listener).unwrap();
        tracing::info!(port = %admin_port, "admin API listening");
        hyper::server::Builder::new(incoming, hyper::server::conn::Http::new())
            .serve(admin.into_make_service())
            .await
            .unwrap();
    });

    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");
    Ok(())
}

fn build_admin_server(
    db: DbPool,
    config: &AppConfig,
    cb: Arc<CircuitBreaker>,
    blacklist: Arc<TokenBlacklist>,
    prometheus_metrics: PrometheusMetrics,
) -> axum::Router {
    use axum::routing::get;

    let auth = api::auth::router(db.clone(), config, blacklist);
    let gateway_api = api::domains::router(db.clone(), config);
    let keys_api = api::keys::router(db.clone(), config);
    let audit_api = api::audit::router(db.clone());
    let metrics_api = api::metrics::router(db.clone(), cb.clone());
    let prom = prometheus_router(prometheus_metrics);
    let traffic_api = api::traffic::router(db.clone());
    let k8s_api = api::k8s::router(db.clone(), cb.clone());
    let upstreams_api = api::upstreams::router(db.clone(), config);
    let rewrite_api = api::rewrite::router(db.clone());
    let health = Router::new().route("/health", get(|| async { "ok" }));

    // Static files for frontend (serve from frontend/dist)
    let static_files = Router::new().nest_service("/", ServeDir::new("frontend/dist"));

    // API routes group
    let api_router = axum::Router::new()
        .merge(auth)
        .merge(keys_api)
        .merge(gateway_api)
        .merge(audit_api)
        .merge(metrics_api)
        .merge(prom)
        .merge(traffic_api)
        .merge(k8s_api)
        .merge(upstreams_api)
        .merge(rewrite_api);

    axum::Router::new()
        .merge(health)
        .merge(api_router)
        .merge(static_files)
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
