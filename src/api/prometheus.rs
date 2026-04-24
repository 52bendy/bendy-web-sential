use axum::{
    extract::State,
    routing::get,
    Router,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct PrometheusMetrics {
    pub requests_total: Arc<AtomicU64>,
    pub requests_success: Arc<AtomicU64>,
    pub requests_failure: Arc<AtomicU64>,
    pub requests_rate_limited: Arc<AtomicU64>,
    pub requests_circuit_breaker: Arc<AtomicU64>,
    pub requests_latency_ms: Arc<AtomicU64>,
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_success: Arc::new(AtomicU64::new(0)),
            requests_failure: Arc::new(AtomicU64::new(0)),
            requests_rate_limited: Arc::new(AtomicU64::new(0)),
            requests_circuit_breaker: Arc::new(AtomicU64::new(0)),
            requests_latency_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn inc_requests(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_success(&self) {
        self.requests_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_failure(&self) {
        self.requests_failure.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_rate_limited(&self) {
        self.requests_rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_circuit_breaker(&self) {
        self.requests_circuit_breaker.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_latency(&self, ms: u64) {
        self.requests_latency_ms.fetch_add(ms, Ordering::Relaxed);
    }
}

#[derive(Clone)]
pub struct PrometheusState {
    pub metrics: PrometheusMetrics,
}

pub fn router(metrics: PrometheusMetrics) -> Router {
    let state = PrometheusState { metrics };
    Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

pub async fn metrics_handler(
    State(state): State<PrometheusState>,
) -> (axum::http::StatusCode, String) {
    let m = &state.metrics;
    let output = format!(
        r#"# HELP bws_requests_total Total number of requests
# TYPE bws_requests_total counter
bws_requests_total {}

# HELP bws_requests_success Total successful requests
# TYPE bws_requests_success counter
bws_requests_success {}

# HELP bws_requests_failure Total failed requests
# TYPE bws_requests_failure counter
bws_requests_failure {}

# HELP bws_requests_rate_limited Total rate-limited requests
# TYPE bws_requests_rate_limited counter
bws_requests_rate_limited {}

# HELP bws_requests_circuit_breaker Circuit breaker rejections
# TYPE bws_requests_circuit_breaker counter
bws_requests_circuit_breaker {}

# HELP bws_requests_latency_ms Total request latency in ms
# TYPE bws_requests_latency_ms counter
bws_requests_latency_ms {}
"#,
        m.requests_total.load(Ordering::Relaxed),
        m.requests_success.load(Ordering::Relaxed),
        m.requests_failure.load(Ordering::Relaxed),
        m.requests_rate_limited.load(Ordering::Relaxed),
        m.requests_circuit_breaker.load(Ordering::Relaxed),
        m.requests_latency_ms.load(Ordering::Relaxed),
    );
    (axum::http::StatusCode::OK, output)
}
