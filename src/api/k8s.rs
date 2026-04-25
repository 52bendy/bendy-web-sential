use axum::{
    extract::State,
    routing::{get, post},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo;

use crate::db::DbPool;
use crate::types::ApiResponse;
use crate::error::AppError;
use crate::middleware::circuit_breaker::{CircuitBreaker, CircuitState};

pub fn router(db: DbPool, circuit_breaker: Arc<CircuitBreaker>) -> Router {
    let state = K8sState { db, circuit_breaker };
    Router::new()
        .route("/api/v1/k8s/health", get(get_health))
        .route("/api/v1/k8s/probe", post(handle_probe))
        .with_state(state)
}

#[derive(Clone)]
pub struct K8sState {
    pub db: DbPool,
    pub circuit_breaker: Arc<CircuitBreaker>,
}

#[derive(Debug, Clone, Serialize)]
pub struct K8sHealthData {
    pub status: String,              // "healthy" | "degraded" | "unhealthy"
    pub uptime_seconds: u64,
    pub memory_usage_bytes: u64,
    pub memory_total_bytes: u64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug, Deserialize)]
pub struct ProbeRequest {
    #[serde(rename = "type")]
    pub probe_type: String,  // "liveness" | "readiness" | "startup"
}

/// GET /api/v1/k8s/health
/// Returns the health status of the container including resource usage
pub async fn get_health(
    State(_state): State<K8sState>,
) -> Result<Json<ApiResponse<K8sHealthData>>, AppError> {
    let (memory_usage, memory_total, cpu_usage) = get_system_metrics()?;
    let uptime = get_uptime_seconds();
    
    // Determine health status based on resource usage
    let memory_percent = if memory_total > 0 {
        (memory_usage as f64 / memory_total as f64) * 100.0
    } else {
        0.0
    };
    
    let status = if memory_percent >= 95.0 || cpu_usage >= 95.0 {
        "unhealthy"
    } else if memory_percent >= 80.0 || cpu_usage >= 80.0 {
        "degraded"
    } else {
        "healthy"
    };
    
    let health_data = K8sHealthData {
        status: status.to_string(),
        uptime_seconds: uptime,
        memory_usage_bytes: memory_usage,
        memory_total_bytes: memory_total,
        cpu_usage_percent: cpu_usage,
    };
    
    Ok(Json(ApiResponse::success(health_data)))
}

/// POST /api/v1/k8s/probe
/// Handles K8s liveness, readiness, and startup probes
#[axum::debug_handler]
pub async fn handle_probe(
    State(state): State<K8sState>,
    Json(req): Json<ProbeRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    match req.probe_type.as_str() {
        "liveness" => {
            tracing::debug!("liveness probe passed");
            Ok(Json(ApiResponse::ok()))
        }
        "readiness" => {
            check_readiness(&state).await?;
            Ok(Json(ApiResponse::ok()))
        }
        "startup" => {
            tracing::debug!("startup probe passed");
            Ok(Json(ApiResponse::ok()))
        }
        _ => Err(AppError::InvalidParam),
    }
}

/// Check readiness: DB connection + circuit breaker state
async fn check_readiness(state: &K8sState) -> Result<(), AppError> {
    // 1. Check DB connection
    {
        let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
        conn.execute("SELECT 1", [])?;
    }

    // 2. Check Circuit Breaker state
    let cb_metrics = state.circuit_breaker.metrics().await;
    if cb_metrics.state == CircuitState::Open {
        tracing::warn!("readiness probe: circuit breaker is open");
        return Err(AppError::ServiceUnavailable);
    }

    Ok(())
}

/// Get system metrics using sysinfo
fn get_system_metrics() -> Result<(u64, u64, f64), AppError> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let memory_usage = sys.used_memory();
    let memory_total = sys.total_memory();
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;

    Ok((memory_usage, memory_total, cpu_usage))
}

/// Get uptime in seconds since process start
fn get_uptime_seconds() -> u64 {
    // Get current process PID using libc
    let pid = unsafe { libc::getpid() };
    let pid = sysinfo::Pid::from_u32(pid as u32);
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes();

    sys.process(pid)
        .map(|p| p.run_time())
        .unwrap_or(0)
}
