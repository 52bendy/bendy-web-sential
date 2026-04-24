use axum::{
    extract::State,
    routing::{get},
    Router, Json,
};
use crate::db::DbPool;
use crate::types::{ApiResponse, AuditLog};
use crate::error::AppError;

pub fn router(db: DbPool) -> Router {
    let state = AuditState { db };
    Router::new()
        .route("/api/v1/audit-logs", get(list_audit_logs))
        .with_state(state)
}

#[derive(Clone)]
pub struct AuditState {
    pub db: DbPool,
}

pub async fn list_audit_logs(
    State(state): State<AuditState>,
) -> Result<Json<ApiResponse<Vec<AuditLog>>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, username, action, resource, resource_id, ip_address, user_agent, details, created_at
         FROM bws_audit_log ORDER BY created_at DESC LIMIT 100"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AuditLog {
            id: row.get(0)?,
            user_id: row.get(1)?,
            username: row.get(2)?,
            action: row.get(3)?,
            resource: row.get(4)?,
            resource_id: row.get(5)?,
            ip_address: row.get(6)?,
            user_agent: row.get(7)?,
            details: row.get(8)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?).unwrap().with_timezone(&chrono::Utc),
        })
    })?;
    let logs: Vec<_> = rows.filter_map(|r| r.ok()).collect();
    Ok(Json(ApiResponse::success(logs)))
}
