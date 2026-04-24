use axum::{
    extract::State,
    routing::{get, post, put, delete},
    Router, Json,
};
use rusqlite::params;
use chrono::Utc;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use crate::db::DbPool;
use crate::types::{ApiResponse, Domain, Route, RouteAction};
use crate::error::AppError;
use crate::config::AppConfig;

pub fn router(db: DbPool, config: &AppConfig) -> Router {
    let state = GatewayState {
        db,
        cloudflare_api_token: config.cloudflare_api_token.clone(),
        cloudflare_zone_id: config.cloudflare_zone_identifier.clone(),
    };
    Router::new()
        .route("/api/v1/domains", get(list_domains))
        .route("/api/v1/domains", post(create_domain))
        .route("/api/v1/domains/{id}", get(get_domain))
        .route("/api/v1/domains/{id}", put(update_domain))
        .route("/api/v1/domains/{id}", delete(delete_domain))
        .route("/api/v1/routes", get(list_routes))
        .route("/api/v1/routes", post(create_route))
        .route("/api/v1/routes/{id}", put(update_route))
        .route("/api/v1/routes/{id}", delete(delete_route))
        .route("/api/v1/cloudflare/dns", get(list_cf_dns_records))
        .route("/api/v1/cloudflare/dns", post(create_cf_dns_record))
        .route("/api/v1/cloudflare/dns/{record_id}", put(update_cf_dns_record))
        .route("/api/v1/cloudflare/dns/{record_id}", delete(delete_cf_dns_record))
        .with_state(state)
}

#[derive(Clone)]
pub struct GatewayState {
    pub db: DbPool,
    pub cloudflare_api_token: Option<String>,
    pub cloudflare_zone_id: Option<String>,
}

// CloudFlare DNS types
#[derive(Debug, Deserialize, Serialize)]
pub struct CfDnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub proxied: Option<bool>,
    pub ttl: Option<u64>,
    pub priority: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CfCreateDnsRecord {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CfUpdateDnsRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u64>,
}

fn cf_client(api_token: &Option<String>) -> Result<reqwest::Client, AppError> {
    if api_token.is_none() {
        return Err(AppError::ConfigError("CloudFlare API token not configured".into()));
    }
    Ok(reqwest::Client::new())
}

fn cf_headers(api_token: &str) -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {}", api_token).parse().unwrap());
    headers.insert(http::header::CONTENT_TYPE, "application/json".parse().unwrap());
    headers
}

/// GET /api/v1/cloudflare/dns
/// Lists DNS records from CloudFlare
pub async fn list_cf_dns_records(
    State(state): State<GatewayState>,
) -> Result<Json<ApiResponse<Vec<CfDnsRecord>>>, AppError> {
    let zone_id = state.cloudflare_zone_id.as_ref().ok_or(AppError::ConfigError("CloudFlare zone not configured".into()))?;
    let token = state.cloudflare_api_token.as_ref().ok_or(AppError::ConfigError("CloudFlare API token not configured".into()))?;

    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
    let res = reqwest::Client::new()
        .get(&url)
        .headers(cf_headers(token))
        .send()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?;

    let status = res.status();
    let body: serde_json::Value = res.json().await.map_err(|e| AppError::UpstreamError(e.to_string()))?;

    if !status.is_success() {
        let msg = body["errors"].as_array().and_then(|a| a.first()).and_then(|e| e["message"].as_str()).unwrap_or("CloudFlare API error");
        return Err(AppError::UpstreamError(msg.to_string()));
    }

    let records: Vec<CfDnsRecord> = match serde_json::from_value(body["result"].clone()) {
        Ok(v) => v,
        Err(_) => return Err(AppError::InternalError),
    };
    Ok(Json(ApiResponse::success(records)))
}

/// POST /api/v1/cloudflare/dns
/// Creates a DNS record on CloudFlare
pub async fn create_cf_dns_record(
    State(state): State<GatewayState>,
    Json(payload): Json<CfCreateDnsRecord>,
) -> Result<Json<ApiResponse<CfDnsRecord>>, AppError> {
    let zone_id = state.cloudflare_zone_id.as_ref().ok_or(AppError::ConfigError("CloudFlare zone not configured".into()))?;
    let token = state.cloudflare_api_token.as_ref().ok_or(AppError::ConfigError("CloudFlare API token not configured".into()))?;

    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
    let res = reqwest::Client::new()
        .post(&url)
        .headers(cf_headers(token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?;

    let status = res.status();
    let body: serde_json::Value = res.json().await.map_err(|e| AppError::UpstreamError(e.to_string()))?;

    if !status.is_success() {
        let msg = body["errors"].as_array().and_then(|a| a.first()).and_then(|e| e["message"].as_str()).unwrap_or("CloudFlare API error");
        return Err(AppError::UpstreamError(msg.to_string()));
    }

    let record: CfDnsRecord = match serde_json::from_value(body["result"].clone()) {
        Ok(v) => v,
        Err(_) => return Err(AppError::InternalError),
    };
    tracing::info!(record_id = %record.id, record_type = %record.record_type, action = "cloudflare_create_dns", "audit");
    Ok(Json(ApiResponse::success(record)))
}

/// PUT /api/v1/cloudflare/dns/{record_id}
/// Updates a DNS record on CloudFlare
pub async fn update_cf_dns_record(
    State(state): State<GatewayState>,
    axum::extract::Path(record_id): axum::extract::Path<String>,
    Json(payload): Json<CfUpdateDnsRecord>,
) -> Result<Json<ApiResponse<CfDnsRecord>>, AppError> {
    let zone_id = state.cloudflare_zone_id.as_ref().ok_or(AppError::ConfigError("CloudFlare zone not configured".into()))?;
    let token = state.cloudflare_api_token.as_ref().ok_or(AppError::ConfigError("CloudFlare API token not configured".into()))?;

    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, record_id);
    let res = reqwest::Client::new()
        .put(&url)
        .headers(cf_headers(token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?;

    let status = res.status();
    let body: serde_json::Value = res.json().await.map_err(|e| AppError::UpstreamError(e.to_string()))?;

    if !status.is_success() {
        let msg = body["errors"].as_array().and_then(|a| a.first()).and_then(|e| e["message"].as_str()).unwrap_or("CloudFlare API error");
        return Err(AppError::UpstreamError(msg.to_string()));
    }

    let record: CfDnsRecord = match serde_json::from_value(body["result"].clone()) {
        Ok(v) => v,
        Err(_) => return Err(AppError::InternalError),
    };
    tracing::info!(record_id = %record.id, action = "cloudflare_update_dns", "audit");
    Ok(Json(ApiResponse::success(record)))
}

/// DELETE /api/v1/cloudflare/dns/{record_id}
/// Deletes a DNS record from CloudFlare
pub async fn delete_cf_dns_record(
    State(state): State<GatewayState>,
    axum::extract::Path(record_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let zone_id = state.cloudflare_zone_id.as_ref().ok_or(AppError::ConfigError("CloudFlare zone not configured".into()))?;
    let token = state.cloudflare_api_token.as_ref().ok_or(AppError::ConfigError("CloudFlare API token not configured".into()))?;

    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, record_id);
    let res = reqwest::Client::new()
        .delete(&url)
        .headers(cf_headers(token))
        .send()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?;

    let status = res.status();
    let body: serde_json::Value = res.json().await.map_err(|e| AppError::UpstreamError(e.to_string()))?;

    if !status.is_success() {
        let msg = body["errors"].as_array().and_then(|a| a.first()).and_then(|e| e["message"].as_str()).unwrap_or("CloudFlare API error");
        return Err(AppError::UpstreamError(msg.to_string()));
    }

    tracing::info!(record_id = %record_id, action = "cloudflare_delete_dns", "audit");
    Ok(Json(ApiResponse::ok()))
}

pub async fn list_domains(State(state): State<GatewayState>) -> Result<Json<ApiResponse<Vec<Domain>>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let mut stmt = conn.prepare(
        "SELECT id, domain, description, hosting_service, active, created_at, updated_at FROM bws_domains ORDER BY domain"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Domain {
            id: row.get(0)?,
            domain: row.get(1)?,
            description: row.get(2)?,
            hosting_service: row.get(3)?,
            active: row.get::<_, i32>(4)? == 1,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?).unwrap().with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?).unwrap().with_timezone(&Utc),
        })
    })?;
    let domains: Vec<_> = rows.filter_map(|r| r.ok()).collect();
    Ok(Json(ApiResponse::success(domains)))
}

pub async fn create_domain(
    State(state): State<GatewayState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Domain>>, AppError> {
    let domain = payload["domain"].as_str().unwrap_or("").to_string();
    let description = payload["description"].as_str().map(|s| s.to_string());
    let hosting_service = payload["hosting_service"].as_str().map(|s| s.to_string());
    let now = Utc::now().to_rfc3339();

    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "INSERT INTO bws_domains (domain, description, hosting_service, active, created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, ?4)",
        params![domain, description, hosting_service, now],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);

    let domain = get_domain_by_id(&state.db, id)?;
    tracing::info!(domain = ?domain.domain, action = "create_domain", "audit");
    Ok(Json(ApiResponse::success(domain)))
}

pub async fn get_domain(
    State(_state): State<GatewayState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<Domain>>, AppError> {
    let domain = get_domain_by_id(&_state.db, id)?;
    Ok(Json(ApiResponse::success(domain)))
}

pub async fn update_domain(
    State(_state): State<GatewayState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Domain>>, AppError> {
    let domain = payload["domain"].as_str().unwrap_or("").to_string();
    let description = payload["description"].as_str().map(|s| s.to_string());
    let hosting_service = payload["hosting_service"].as_str().map(|s| s.to_string());
    let active = payload["active"].as_i64().unwrap_or(1) == 1;
    let now = Utc::now().to_rfc3339();

    let conn = _state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "UPDATE bws_domains SET domain = ?1, description = ?2, hosting_service = ?3, active = ?4, updated_at = ?5 WHERE id = ?6",
        params![domain, description, hosting_service, active as i32, now, id],
    )?;
    drop(conn);

    let domain = get_domain_by_id(&_state.db, id)?;
    tracing::info!(id = %id, action = "update_domain", "audit");
    Ok(Json(ApiResponse::success(domain)))
}

pub async fn delete_domain(
    State(_state): State<GatewayState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let conn = _state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute("DELETE FROM bws_domains WHERE id = ?1", params![id])?;
    drop(conn);
    tracing::info!(id = %id, action = "delete_domain", "audit");
    Ok(Json(ApiResponse::ok()))
}

pub async fn list_routes(State(state): State<GatewayState>) -> Result<Json<ApiResponse<Vec<Route>>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let mut stmt = conn.prepare(
        "SELECT id, domain_id, path_pattern, action, target, description, priority, active, created_at, updated_at FROM bws_routes ORDER BY domain_id, priority DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        let action_str: String = row.get(3)?;
        let action = match action_str.as_str() {
            "redirect" => RouteAction::Redirect,
            "static" => RouteAction::Static,
            _ => RouteAction::Proxy,
        };
        Ok(Route {
            id: row.get(0)?,
            domain_id: row.get(1)?,
            path_pattern: row.get(2)?,
            action,
            target: row.get(4)?,
            description: row.get(5)?,
            priority: row.get(6)?,
            active: row.get::<_, i32>(7)? == 1,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?).unwrap().with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?).unwrap().with_timezone(&Utc),
        })
    })?;
    let routes: Vec<_> = rows.filter_map(|r| r.ok()).collect();
    Ok(Json(ApiResponse::success(routes)))
}

pub async fn create_route(
    State(state): State<GatewayState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Route>>, AppError> {
    let domain_id = payload["domain_id"].as_i64().ok_or(AppError::InvalidParam)?;
    let path_pattern = payload["path_pattern"].as_str().unwrap_or("").to_string();
    let action = payload["action"].as_str().unwrap_or("proxy").to_string();
    let target = payload["target"].as_str().unwrap_or("").to_string();
    let description = payload["description"].as_str().map(|s| s.to_string());
    let priority = payload["priority"].as_i64().unwrap_or(0) as i32;
    let now = Utc::now().to_rfc3339();

    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "INSERT INTO bws_routes (domain_id, path_pattern, action, target, description, priority, active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)",
        params![domain_id, path_pattern, action, target, description, priority, now],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);

    let route = get_route_by_id(&state.db, id)?;
    tracing::info!(id = %id, action = "create_route", "audit");
    Ok(Json(ApiResponse::success(route)))
}

pub async fn update_route(
    State(_state): State<GatewayState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Route>>, AppError> {
    let path_pattern = payload["path_pattern"].as_str().unwrap_or("").to_string();
    let action = payload["action"].as_str().unwrap_or("proxy").to_string();
    let target = payload["target"].as_str().unwrap_or("").to_string();
    let description = payload["description"].as_str().map(|s| s.to_string());
    let priority = payload["priority"].as_i64().unwrap_or(0) as i32;
    let active = payload["active"].as_i64().unwrap_or(1) == 1;
    let now = Utc::now().to_rfc3339();

    let conn = _state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "UPDATE bws_routes SET path_pattern = ?1, action = ?2, target = ?3, description = ?4, priority = ?5, active = ?6, updated_at = ?7 WHERE id = ?8",
        params![path_pattern, action, target, description, priority, active as i32, now, id],
    )?;
    drop(conn);

    let route = get_route_by_id(&_state.db, id)?;
    tracing::info!(id = %id, action = "update_route", "audit");
    Ok(Json(ApiResponse::success(route)))
}

pub async fn delete_route(
    State(_state): State<GatewayState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let conn = _state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute("DELETE FROM bws_routes WHERE id = ?1", params![id])?;
    drop(conn);
    tracing::info!(id = %id, action = "delete_route", "audit");
    Ok(Json(ApiResponse::ok()))
}

fn get_domain_by_id(db: &DbPool, id: i64) -> Result<Domain, AppError> {
    let conn = db.lock().map_err(|_| AppError::InternalError)?;
    let d = conn.query_row(
        "SELECT id, domain, description, hosting_service, active, created_at, updated_at FROM bws_domains WHERE id = ?1",
        params![id],
        |row| {
            Ok(Domain {
                id: row.get(0)?,
                domain: row.get(1)?,
                description: row.get(2)?,
                hosting_service: row.get(3)?,
                active: row.get::<_, i32>(4)? == 1,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?).unwrap().with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?).unwrap().with_timezone(&Utc),
            })
        },
    )?;
    Ok(d)
}

fn get_route_by_id(db: &DbPool, id: i64) -> Result<Route, AppError> {
    let conn = db.lock().map_err(|_| AppError::InternalError)?;
    let r = conn.query_row(
        "SELECT id, domain_id, path_pattern, action, target, description, priority, active, created_at, updated_at FROM bws_routes WHERE id = ?1",
        params![id],
        |row| {
            let action_str: String = row.get(3)?;
            let action = match action_str.as_str() {
                "redirect" => RouteAction::Redirect,
                "static" => RouteAction::Static,
                _ => RouteAction::Proxy,
            };
            Ok(Route {
                id: row.get(0)?,
                domain_id: row.get(1)?,
                path_pattern: row.get(2)?,
                action,
                target: row.get(4)?,
                description: row.get(5)?,
                priority: row.get(6)?,
                active: row.get::<_, i32>(7)? == 1,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?).unwrap().with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?).unwrap().with_timezone(&Utc),
            })
        },
    )?;
    Ok(r)
}
