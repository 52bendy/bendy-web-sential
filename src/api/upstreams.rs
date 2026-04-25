use axum::{
    extract::State,
    routing::{get, post, put, delete},
    Router, Json,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use crate::types::{ApiResponse, Upstream};
use crate::error::AppError;
use crate::config::AppConfig;
use chrono::Utc;

pub fn router(db: DbPool, _config: &AppConfig) -> Router {
    let state = UpstreamState { db };
    Router::new()
        .route("/api/v1/upstreams", get(list_upstreams))
        .route("/api/v1/upstreams", post(create_upstream))
        .route("/api/v1/upstreams/:id", get(get_upstream))
        .route("/api/v1/upstreams/:id", put(update_upstream))
        .route("/api/v1/upstreams/:id", delete(delete_upstream))
        .route("/api/v1/upstreams/route/:route_id", get(list_by_route))
        .route("/api/v1/upstreams/:id/health", put(update_health))
        .with_state(state)
}

#[derive(Clone)]
pub struct UpstreamState {
    pub db: DbPool,
}

#[derive(Debug, Deserialize)]
pub struct CreateUpstream {
    pub route_id: i64,
    pub target_url: String,
    pub weight: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUpstream {
    #[serde(default)]
    pub target_url: Option<String>,
    #[serde(default)]
    pub weight: Option<i32>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub healthy: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpstreamListResponse {
    pub upstreams: Vec<Upstream>,
    pub total: usize,
}

/// GET /api/v1/upstreams
async fn list_upstreams(
    State(state): State<UpstreamState>,
) -> Result<Json<ApiResponse<UpstreamListResponse>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let mut stmt = conn.prepare(
        "SELECT id, route_id, target_url, weight, active, healthy, created_at FROM bws_upstreams ORDER BY id"
    )?;

    let upstreams = stmt.query_map([], |row| {
        Ok(Upstream {
            id: row.get(0)?,
            route_id: row.get(1)?,
            target_url: row.get(2)?,
            weight: row.get(3)?,
            active: row.get::<_, i32>(4)? != 0,
            healthy: row.get::<_, i32>(5)? != 0,
            created_at: row.get::<_, String>(6)?
                .parse()
                .unwrap_or_else(|_| Utc::now()),
        })
    })?.collect::<Result<Vec<_>, _>>().map_err(|_| AppError::InternalError)?;

    Ok(Json(ApiResponse::success(UpstreamListResponse {
        total: upstreams.len(),
        upstreams,
    })))
}

/// POST /api/v1/upstreams
async fn create_upstream(
    State(state): State<UpstreamState>,
    Json(payload): Json<CreateUpstream>,
) -> Result<Json<ApiResponse<Upstream>>, AppError> {
    // Validate target_url
    if payload.target_url.is_empty() {
        return Err(AppError::BadRequest("target_url is required".into()));
    }
    if !payload.target_url.starts_with("http://") && !payload.target_url.starts_with("https://") {
        return Err(AppError::BadRequest("target_url must start with http:// or https://".into()));
    }

    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    // Verify route exists
    let route_exists: bool = conn.query_row(
        "SELECT 1 FROM bws_routes WHERE id = ?1",
        params![payload.route_id],
        |_| Ok(true)
    ).unwrap_or(false);

    if !route_exists {
        return Err(AppError::NotFound);
    }

    let weight = payload.weight.unwrap_or(1);
    let active = payload.active.unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO bws_upstreams (route_id, target_url, weight, active, healthy, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        params![payload.route_id, payload.target_url, weight, active as i32, now]
    )?;

    let id = conn.last_insert_rowid();

    Ok(Json(ApiResponse::success(Upstream {
        id,
        route_id: payload.route_id,
        target_url: payload.target_url,
        weight,
        active,
        healthy: true,
        created_at: now.parse().unwrap_or_else(|_| Utc::now()),
    })))
}

/// GET /api/v1/upstreams/:id
async fn get_upstream(
    State(state): State<UpstreamState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<Upstream>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let upstream = conn.query_row(
        "SELECT id, route_id, target_url, weight, active, healthy, created_at FROM bws_upstreams WHERE id = ?1",
        params![id],
        |row| {
            Ok(Upstream {
                id: row.get(0)?,
                route_id: row.get(1)?,
                target_url: row.get(2)?,
                weight: row.get(3)?,
                active: row.get::<_, i32>(4)? != 0,
                healthy: row.get::<_, i32>(5)? != 0,
                created_at: row.get::<_, String>(6)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        }
    ).map_err(|_| AppError::NotFound)?;

    Ok(Json(ApiResponse::success(upstream)))
}

/// PUT /api/v1/upstreams/:id
async fn update_upstream(
    State(state): State<UpstreamState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<UpdateUpstream>,
) -> Result<Json<ApiResponse<Upstream>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    // Check exists
    let current = conn.query_row(
        "SELECT id, route_id, target_url, weight, active, healthy, created_at FROM bws_upstreams WHERE id = ?1",
        params![id],
        |row| {
            Ok(Upstream {
                id: row.get(0)?,
                route_id: row.get(1)?,
                target_url: row.get(2)?,
                weight: row.get(3)?,
                active: row.get::<_, i32>(4)? != 0,
                healthy: row.get::<_, i32>(5)? != 0,
                created_at: row.get::<_, String>(6)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        }
    ).map_err(|_| AppError::NotFound)?;

    // Build update query
    let target_url = payload.target_url.unwrap_or(current.target_url);
    let weight = payload.weight.unwrap_or(current.weight);
    let active = payload.active.unwrap_or(current.active);
    let healthy = payload.healthy.unwrap_or(current.healthy);

    // Validate target_url if changed
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return Err(AppError::BadRequest("target_url must start with http:// or https://".into()));
    }

    conn.execute(
        "UPDATE bws_upstreams SET target_url = ?1, weight = ?2, active = ?3, healthy = ?4 WHERE id = ?5",
        params![target_url, weight, active as i32, healthy as i32, id]
    )?;

    Ok(Json(ApiResponse::success(Upstream {
        id,
        route_id: current.route_id,
        target_url,
        weight,
        active,
        healthy,
        created_at: current.created_at,
    })))
}

/// DELETE /api/v1/upstreams/:id
async fn delete_upstream(
    State(state): State<UpstreamState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let rows = conn.execute(
        "DELETE FROM bws_upstreams WHERE id = ?1",
        params![id]
    )?;

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(ApiResponse::ok()))
}

/// GET /api/v1/upstreams/route/:route_id
async fn list_by_route(
    State(state): State<UpstreamState>,
    axum::extract::Path(route_id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<UpstreamListResponse>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let mut stmt = conn.prepare(
        "SELECT id, route_id, target_url, weight, active, healthy, created_at FROM bws_upstreams WHERE route_id = ?1 ORDER BY id"
    )?;

    let upstreams = stmt.query_map(params![route_id], |row| {
        Ok(Upstream {
            id: row.get(0)?,
            route_id: row.get(1)?,
            target_url: row.get(2)?,
            weight: row.get(3)?,
            active: row.get::<_, i32>(4)? != 0,
            healthy: row.get::<_, i32>(5)? != 0,
            created_at: row.get::<_, String>(6)?
                .parse()
                .unwrap_or_else(|_| Utc::now()),
        })
    })?.collect::<Result<Vec<_>, _>>().map_err(|_| AppError::InternalError)?;

    Ok(Json(ApiResponse::success(UpstreamListResponse {
        total: upstreams.len(),
        upstreams,
    })))
}

/// PUT /api/v1/upstreams/:id/health
async fn update_health(
    State(state): State<UpstreamState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Upstream>>, AppError> {
    let healthy = payload.get("healthy")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| AppError::BadRequest("healthy field is required".into()))?;

    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let rows = conn.execute(
        "UPDATE bws_upstreams SET healthy = ?1 WHERE id = ?2",
        params![healthy as i32, id]
    )?;

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    // Fetch updated record
    let upstream = conn.query_row(
        "SELECT id, route_id, target_url, weight, active, healthy, created_at FROM bws_upstreams WHERE id = ?1",
        params![id],
        |row| {
            Ok(Upstream {
                id: row.get(0)?,
                route_id: row.get(1)?,
                target_url: row.get(2)?,
                weight: row.get(3)?,
                active: row.get::<_, i32>(4)? != 0,
                healthy: row.get::<_, i32>(5)? != 0,
                created_at: row.get::<_, String>(6)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        }
    ).map_err(|_| AppError::InternalError)?;

    Ok(Json(ApiResponse::success(upstream)))
}
