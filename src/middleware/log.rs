use axum::{
    middleware::Next,
    response::Response,
    body::Body,
};
use http::Request;
use std::time::Instant;

pub async fn request_log(req: Request<Body>, next: Next<Body>) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let host = req.headers().get("host").and_then(|v| v.to_str().ok()).unwrap_or("?").to_string();

    let res = next.run(req).await;
    let elapsed = start.elapsed();
    let status = res.status().as_u16();

    tracing::info!(
        method = %method,
        path = %path,
        host = %host,
        status = %status,
        duration_ms = elapsed.as_millis() as u64,
        "http_request"
    );

    res
}
