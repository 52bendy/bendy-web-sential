use axum::{
    extract::State,
    routing::{get},
    Router, Json,
};
use serde::Serialize;
use crate::db::DbPool;
use crate::types::ApiResponse;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::error::AppError;
use std::sync::Arc;

#[derive(Clone)]
pub struct MetricsState {
    pub db: DbPool,
    pub circuit_breaker: Arc<CircuitBreaker>,
}

pub fn router(db: DbPool, cb: Arc<CircuitBreaker>) -> Router {
    let state = MetricsState { db, circuit_breaker: cb };
    Router::new()
        .route("/api/v1/metrics", get(get_metrics))
        .with_state(state)
}

#[derive(Serialize)]
pub struct MetricsData {
    pub total_requests: u32,
    pub active_routes: i64,
    pub domains_count: i64,
    pub circuit_breaker_state: String,
}

pub async fn get_metrics(
    State(state): State<MetricsState>,
) -> Result<Json<ApiResponse<MetricsData>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let active_routes: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bws_routes WHERE active = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    let domains_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bws_domains WHERE active = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    let cb_metrics = state.circuit_breaker.metrics();
    let cb_state = match cb_metrics.state {
        crate::middleware::circuit_breaker::CircuitState::Closed => "Closed",
        crate::middleware::circuit_breaker::CircuitState::Open => "Open",
        crate::middleware::circuit_breaker::CircuitState::HalfOpen => "HalfOpen",
    };

    let metrics = MetricsData {
        total_requests: cb_metrics.failure_count + cb_metrics.success_count,
        active_routes,
        domains_count,
        circuit_breaker_state: cb_state.to_string(),
    };

    Ok(Json(ApiResponse::success(metrics)))
}
