use axum::{
    extract::State,
    routing::{get, post, delete},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use rusqlite::params;

use crate::db::DbPool;
use crate::types::{ApiResponse, RewriteRule};
use crate::error::AppError;

pub fn router(db: DbPool) -> Router {
    Router::new()
        .route("/api/rewrites", get(list_rewrites))
        .route("/api/rewrites", post(create_rewrite))
        .route("/api/rewrites/:id", delete(delete_rewrite))
        .with_state(db)
}

#[derive(Debug, Serialize)]
pub struct RewriteRuleResponse {
    pub id: i64,
    pub name: String,
    pub rule_type: String,
    pub pattern: String,
    pub replacement: String,
    pub enabled: bool,
}

impl From<RewriteRule> for RewriteRuleResponse {
    fn from(r: RewriteRule) -> Self {
        Self {
            id: r.id,
            name: r.name,
            rule_type: r.rule_type,
            pattern: r.pattern,
            replacement: r.replacement,
            enabled: r.enabled,
        }
    }
}

/// GET /api/rewrites - List all rewrite rules
pub async fn list_rewrites(
    State(db): State<DbPool>,
) -> Result<Json<ApiResponse<Vec<RewriteRuleResponse>>>, AppError> {
    let conn = db.lock().map_err(|_| AppError::InternalError)?;

    let mut stmt = conn.prepare(
        "SELECT id, name, rule_type, pattern, replacement, enabled FROM bws_rewrite_rules ORDER BY id"
    )?;

    let rules = stmt.query_map([], |row| {
        Ok(RewriteRuleResponse {
            id: row.get(0)?,
            name: row.get(1)?,
            rule_type: row.get(2)?,
            pattern: row.get(3)?,
            replacement: row.get(4)?,
            enabled: row.get::<_, i32>(5)? != 0,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(Json(ApiResponse::success(rules)))
}

/// POST /api/rewrites - Create a new rewrite rule
pub async fn create_rewrite(
    State(db): State<DbPool>,
    Json(req): Json<CreateRewriteRequest>,
) -> Result<Json<ApiResponse<RewriteRuleResponse>>, AppError> {
    if req.name.is_empty() || req.pattern.is_empty() {
        return Err(AppError::InvalidParam);
    }

    let conn = db.lock().map_err(|_| AppError::InternalError)?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO bws_rewrite_rules (name, rule_type, pattern, replacement, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![req.name, req.rule_type, req.pattern, req.replacement, req.enabled as i32, now, now],
    )?;

    let id = conn.last_insert_rowid();

    Ok(Json(ApiResponse::success(RewriteRuleResponse {
        id,
        name: req.name,
        rule_type: req.rule_type,
        pattern: req.pattern,
        replacement: req.replacement,
        enabled: req.enabled,
    })))
}

/// DELETE /api/rewrites/:id
pub async fn delete_rewrite(
    State(db): State<DbPool>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let conn = db.lock().map_err(|_| AppError::InternalError)?;
    let affected = conn.execute("DELETE FROM bws_rewrite_rules WHERE id = ?1", params![id])?;

    if affected == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(ApiResponse::ok()))
}

#[derive(Debug, Deserialize)]
pub struct CreateRewriteRequest {
    pub name: String,
    pub rule_type: String,
    pub pattern: String,
    pub replacement: String,
    pub enabled: bool,
}
