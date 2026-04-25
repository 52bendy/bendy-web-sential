//! API Key Management
//! Provides CRUD operations for API keys

use axum::{
    extract::State,
    routing::{get, post, delete},
    Router, Json,
};
use rusqlite::params;
use chrono::Utc;

use crate::db::DbPool;
use crate::types::ApiResponse;
use crate::error::AppError;
use crate::config::AppConfig;
use crate::middleware::auth::{create_api_key, revoke_api_key, list_api_keys, ApiKeyInfo};

#[derive(Clone)]
pub struct ApiKeyState {
    pub db: DbPool,
}

pub fn router(db: DbPool, _config: &AppConfig) -> Router {
    let state = ApiKeyState { db };
    Router::new()
        .route("/api/v1/keys", get(list_keys))
        .route("/api/v1/keys", post(create_key))
        .route("/api/v1/keys/{id}", delete(delete_key))
        .with_state(state)
}

#[derive(serde::Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub role: String,
    pub expires_at: Option<String>, // ISO 8601 datetime
}

#[derive(serde::Serialize)]
pub struct CreateApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub key: String, // Only returned once on creation
    pub role: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// GET /api/v1/keys
/// List all API keys
pub async fn list_keys(
    State(state): State<ApiKeyState>,
) -> Result<Json<ApiResponse<Vec<ApiKeyInfo>>>, AppError> {
    let keys = list_api_keys(&state.db)?;
    Ok(Json(ApiResponse::success(keys)))
}

/// POST /api/v1/keys
/// Create a new API key
pub async fn create_key(
    State(state): State<ApiKeyState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiResponse<CreateApiKeyResponse>>, AppError> {
    // Validate role
    let valid_roles = ["user", "admin", "superadmin"];
    if !valid_roles.contains(&payload.role.as_str()) {
        return Err(AppError::InvalidParam);
    }

    // Create the API key
    let (raw_key, key_hash) = create_api_key(
        &state.db,
        &payload.name,
        &payload.role,
        payload.expires_at.as_deref(),
    )?;

    // Get the created key info
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let id: i64 = conn.query_row(
        "SELECT id FROM bws_api_keys WHERE key_hash = ?1",
        [&key_hash],
        |row| row.get(0),
    )?;
    let now = Utc::now().to_rfc3339();

    tracing::info!(id = %id, name = %payload.name, role = %payload.role, action = "create_api_key", "audit");

    Ok(Json(ApiResponse::success(CreateApiKeyResponse {
        id,
        name: payload.name,
        key: raw_key, // Only returned on creation
        role: payload.role,
        created_at: now,
        expires_at: payload.expires_at,
    })))
}

/// DELETE /api/v1/keys/{id}
/// Revoke an API key by ID
pub async fn delete_key(
    State(state): State<ApiKeyState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    // Get the key hash first
    let key_hash: String = conn.query_row(
        "SELECT key_hash FROM bws_api_keys WHERE id = ?1",
        params![id],
        |row| row.get(0),
    ).map_err(|_| AppError::NotFound)?;

    drop(conn);

    revoke_api_key(&state.db, &key_hash)?;

    tracing::info!(id = %id, action = "revoke_api_key", "audit");
    Ok(Json(ApiResponse::ok()))
}
