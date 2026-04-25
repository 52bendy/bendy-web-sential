use std::sync::Arc;
use std::net::IpAddr;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    body::Body,
    response::{IntoResponse, Response},
    Router, Json,
};
use tower_http::trace::TraceLayer;
use axum::extract::FromRef;

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::middleware::ratelimit::RateLimiters;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::retry::RetryClient;
use crate::middleware::auth::{check_route_auth, extract_api_key, extract_bearer_token};
use crate::security::JwtServiceClone;
use crate::types::{RouteWithAuth, RateLimitDimension, AuthStrategy};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: AppConfig,
    pub rate_limiters: RateLimiters,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub retry_client: Arc<RetryClient>,
    pub jwt: JwtServiceClone,
}

// Implement FromRef for AppState to Arc<AppState>
impl FromRef<AppState> for Arc<CircuitBreaker> {
    fn from_ref(state: &AppState) -> Self {
        state.circuit_breaker.clone()
    }
}

impl FromRef<AppState> for Arc<RetryClient> {
    fn from_ref(state: &AppState) -> Self {
        state.retry_client.clone()
    }
}

impl FromRef<AppState> for JwtServiceClone {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}

pub async fn start_gateway(state: AppState) {
    let port = state.config.gateway_port;
    let app = Router::new()
        .fallback(proxy_handler)
        .with_state(state)
        .layer(TraceLayer::new_for_http());
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
    State(state): State<AppState>,
    req: Request<Body>,
) -> Response {
    let ip = extract_ip(&req);
    let path = req.uri().path().to_string();

    if let Some(err) = crate::middleware::ratelimit::check_rate_limit(&state.rate_limiters, &ip, &path).await {
        return err.into_response();
    }

    let host = req.headers().get("host").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();

    let route_info = find_route_full(&state.db, &host, &path).await;

    if let Some(route) = route_info {
        tracing::debug!(route_id = %route.id, action = %route.action, target = %route.target, host = %host, path = %path, "route matched");

        // Extract auth credentials synchronously before async boundary
        let bearer_token = extract_bearer_token(req.headers());
        let api_key = extract_api_key(req.headers());
        let auth_result = check_route_auth(
            &route.auth_strategy,
            &route.min_role,
            &state.jwt,
            &state.db,
            bearer_token,
            api_key,
        ).await;

        match auth_result {
            Ok(auth) => {
                tracing::debug!(
                    ip = %ip,
                    path = %path,
                    auth_strategy = ?route.auth_strategy,
                    username = ?auth.username,
                    "request authenticated, proceeding to proxy"
                );
            }
            Err(e) => {
                tracing::warn!(ip = %ip, path = %path, auth_strategy = ?route.auth_strategy, error = %e, "authentication failed");
                return e.into_response();
            }
        }

        // Check route-level rate limiting
        if let Some(err) = check_route_ratelimit(&state, &route, &ip).await {
            return err.into_response();
        }

        match route.action.as_str() {
            "proxy" => {
                let uri_path = req.uri().path_and_query().map(|pq| pq.to_string()).unwrap_or_else(|| "/".to_string());
                let url = format!("{}{}", route.target.trim_end_matches('/'), uri_path);

                let headers = req.headers().clone();
                let method = req.method().clone();
                tracing::debug!(url = %url, method = ?method, "proxying request");

                if !state.circuit_breaker.try_acquire().await {
                    tracing::warn!("circuit breaker rejected request");
                    return crate::error::AppError::CircuitBreakerOpen.into_response();
                }

                let resp = state.retry_client.request(method, &url, &headers).await;

                match resp {
                    Ok(resp) => {
                        let upstream_status = resp.status().as_u16();
                        tracing::debug!(status = %upstream_status, "upstream request succeeded");
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
                (StatusCode::FOUND, [("Location", route.target.as_str())], "".as_bytes().to_vec()).into_response()
            }
            "static" => {
                match tokio::fs::read(&route.target).await {
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

/// Find route with full auth and rate limit info
async fn find_route_full(db: &DbPool, host: &str, path: &str) -> Option<RouteWithAuth> {
    let conn = match db.lock() { Ok(c) => c, Err(_) => return None };

    let result = conn.query_row(
        "SELECT r.id, r.action, r.target, r.auth_strategy, r.min_role,
                r.ratelimit_window, r.ratelimit_limit, r.ratelimit_dimension
         FROM bws_routes r
         JOIN bws_domains d ON r.domain_id = d.id
         WHERE d.domain = ?1 AND d.active = 1 AND r.active = 1
         AND ?2 LIKE (r.path_pattern || '%')
         ORDER BY r.priority DESC, r.id DESC
         LIMIT 1",
        [host, path],
        |row| {
            let auth_strategy_str: String = row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "none".into());
            let ratelimit_dimension_str: String = row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "ip".into());

            Ok(RouteWithAuth {
                id: row.get(0)?,
                action: row.get(1)?,
                target: row.get(2)?,
                auth_strategy: AuthStrategy::from_str(&auth_strategy_str),
                min_role: row.get(4)?,
                ratelimit_window: row.get(5)?,
                ratelimit_limit: row.get(6)?,
                ratelimit_dimension: RateLimitDimension::from_str(&ratelimit_dimension_str),
            })
        },
    ).ok();

    result
}

/// Check route-level rate limiting
async fn check_route_ratelimit(
    _state: &AppState,
    route: &RouteWithAuth,
    ip: &IpAddr,
) -> Option<crate::error::AppError> {
    // Skip if no rate limit configured
    let (window, limit, dimension) = match (route.ratelimit_window, route.ratelimit_limit, &route.ratelimit_dimension) {
        (Some(w), Some(l), d) if w > 0 && l > 0 => (w, l, d),
        _ => return None,
    };

    let allowed = match dimension {
        RateLimitDimension::Ip => {
            // For per-IP limiting, use the window in seconds
            // This is a simplified implementation - governor uses per-second by default
            // For proper window support, we'd need a sliding window implementation
            let quota = governor::Quota::per_second(
                std::num::NonZeroU32::new(limit as u32).unwrap()
            );
            let limiter = governor::RateLimiter::dashmap(quota);
            limiter.check_key(ip).is_ok()
        }
        RateLimitDimension::Key => {
            // For key-based limiting, use API key or a default key
            // This would require extracting the API key from the request
            // For now, fall back to IP-based
            let quota = governor::Quota::per_second(
                std::num::NonZeroU32::new(limit as u32).unwrap()
            );
            let limiter = governor::RateLimiter::dashmap(quota);
            limiter.check_key(ip).is_ok()
        }
        RateLimitDimension::Global => {
            // For global route-level limiting
            let quota = governor::Quota::per_second(
                std::num::NonZeroU32::new(limit as u32).unwrap()
            );
            let limiter = governor::RateLimiter::dashmap(quota);
            limiter.check_key(&()).is_ok()
        }
    };

    if !allowed {
        tracing::warn!(
            ip = %ip,
            route_id = %route.id,
            window = %window,
            limit = %limit,
            dimension = ?dimension,
            "route-level rate limit exceeded"
        );
        return Some(crate::error::AppError::RateLimited);
    }

    None
}
