use std::sync::Arc;
use std::net::IpAddr;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    body::Body,
    response::{IntoResponse, Response},
    routing::any,
    Router, Json,
};
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::middleware::ratelimit::RateLimiters;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::retry::RetryClient;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: AppConfig,
    pub rate_limiters: RateLimiters,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub retry_client: Arc<RetryClient>,
}

pub async fn start_gateway(state: Arc<AppState>) {
    let app = Router::new()
        .route("/{*path}", any(proxy_handler))
        .route("/", any(proxy_handler))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());

    let port = state.config.gateway_port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    tracing::info!(port = %port, "gateway listening");
    axum::serve(listener, app).await.unwrap();
}

fn extract_ip(req: &Request<Body>) -> IpAddr {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| {
            req.headers()
                .get("cf-connecting-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse().ok())
        })
        .unwrap_or_else(|| IpAddr::from([0, 0, 0, 0]))
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Response {
    let ip = extract_ip(&req);
    let path = req.uri().path().to_string();

    if let Some(err) = crate::middleware::ratelimit::check_rate_limit(&state.rate_limiters, &ip, &path).await {
        return err.into_response();
    }

    let host = req.headers().get("host").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();

    if let Some((action, target)) = find_route(&state.db, &host, &path).await {
        match action.as_str() {
            "proxy" => {
                let uri_path = req.uri().path_and_query().map(|pq| pq.to_string()).unwrap_or_default();
                let url = format!("{}{}", target.trim_end_matches('/'), uri_path);

                let headers = req.headers().clone();
                let method = req.method().clone();

                if !state.circuit_breaker.try_acquire().await {
                    tracing::warn!("circuit breaker rejected request");
                    return crate::error::AppError::CircuitBreakerOpen.into_response();
                }

                let resp = state.retry_client.request(method, &url, &headers).await;

                match resp {
                    Ok(resp) => {
                        let upstream_status = resp.status().as_u16();
                        let status = StatusCode::from_u16(upstream_status).unwrap_or(StatusCode::OK);
                        let ct = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("text/plain").to_string();
                        let body = resp.bytes().await.unwrap_or_default().to_vec();

                        if upstream_status >= 500 {
                            state.circuit_breaker.record_failure().await;
                        } else {
                            state.circuit_breaker.record_success().await;
                        }

                        (status, [("content-type", ct.as_str())], body).into_response()
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "proxy upstream failed after retries");
                        state.circuit_breaker.record_failure().await;
                        (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"code": 4004, "message": format!("upstream error: {}", e), "data": null}))).into_response()
                    }
                }
            }
            "redirect" => {
                (StatusCode::FOUND, [("Location", target.as_str())], "".as_bytes().to_vec()).into_response()
            }
            "static" => {
                match tokio::fs::read(&target).await {
                    Ok(content) => (StatusCode::OK, [("Content-Type", "text/html; charset=utf-8")], content).into_response(),
                    Err(e) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"code": 3001, "message": format!("static file not found: {}", e), "data": null}))).into_response(),
                }
            }
            _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"code": 3001, "message": "unknown action", "data": null}))).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"code": 3001, "message": "no route found", "data": null}))).into_response()
    }
}

async fn find_route(db: &DbPool, host: &str, path: &str) -> Option<(String, String)> {
    let conn = match db.lock() { Ok(c) => c, Err(_) => return None };

    conn.query_row(
        "SELECT r.action, r.target
         FROM bws_routes r
         JOIN bws_domains d ON r.domain_id = d.id
         WHERE d.domain = ?1 AND d.active = 1 AND r.active = 1
         AND ?2 LIKE (r.path_pattern || '%')
         ORDER BY r.priority DESC
         LIMIT 1",
        [host, path],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    ).ok()
}