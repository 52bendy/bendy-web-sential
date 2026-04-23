use axum::{
    extract::State,
    routing::{get, post, put, delete},
    Router, Json,
};
use rusqlite::params;
use chrono::Utc;
use crate::db::DbPool;
use crate::types::{ApiResponse, Domain, Route, RouteAction};
use crate::error::AppError;

pub fn router(db: DbPool) -> Router {
    let state = GatewayState { db };
    Router::new()
        .route("/api/v1/domains", get(list_domains))
        .route("/api/v1/domains", post(create_domain))
        .route("/api/v1/domains/:id", get(get_domain))
        .route("/api/v1/domains/:id", put(update_domain))
        .route("/api/v1/domains/:id", delete(delete_domain))
        .route("/api/v1/routes", get(list_routes))
        .route("/api/v1/routes", post(create_route))
        .route("/api/v1/routes/:id", put(update_route))
        .route("/api/v1/routes/:id", delete(delete_route))
        .with_state(state)
}

#[derive(Clone)]
pub struct GatewayState {
    pub db: DbPool,
}

pub async fn list_domains(State(state): State<GatewayState>) -> Result<Json<ApiResponse<Vec<Domain>>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let mut stmt = conn.prepare(
        "SELECT id, domain, description, active, created_at, updated_at FROM bws_domains ORDER BY domain"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Domain {
            id: row.get(0)?,
            domain: row.get(1)?,
            description: row.get(2)?,
            active: row.get::<_, i32>(3)? == 1,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?).unwrap().with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?).unwrap().with_timezone(&Utc),
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
    let now = Utc::now().to_rfc3339();

    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "INSERT INTO bws_domains (domain, description, active, created_at, updated_at) VALUES (?1, ?2, 1, ?3, ?3)",
        params![domain, description, now],
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
    let active = payload["active"].as_i64().unwrap_or(1) == 1;
    let now = Utc::now().to_rfc3339();

    let conn = _state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "UPDATE bws_domains SET domain = ?1, description = ?2, active = ?3, updated_at = ?4 WHERE id = ?5",
        params![domain, description, active as i32, now, id],
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
        "SELECT id, domain, description, active, created_at, updated_at FROM bws_domains WHERE id = ?1",
        params![id],
        |row| {
            Ok(Domain {
                id: row.get(0)?,
                domain: row.get(1)?,
                description: row.get(2)?,
                active: row.get::<_, i32>(3)? == 1,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?).unwrap().with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?).unwrap().with_timezone(&Utc),
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
