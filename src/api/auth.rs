use axum::{
    extract::State,
    routing::{post, get},
    Router, Json,
};
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::security::JwtServiceClone;
use crate::types::{ApiResponse, LoginRequest, LoginResponse};
use crate::error::AppError;

#[derive(Clone)]
pub struct AuthState {
    pub db: DbPool,
    pub jwt: JwtServiceClone,
}

pub fn router(db: DbPool, config: &AppConfig) -> Router {
    let jwt = JwtServiceClone::new(config.jwt_secret.clone(), config.jwt_expiry_secs);
    let state = AuthState { db, jwt };

    Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/me", get(me))
        .with_state(state)
}

pub async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let user: Option<(i64, String)> = conn.query_row(
        "SELECT id, password_hash FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [&req.username],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).ok();

    let user = user.ok_or(AppError::InvalidCredentials)?;

    if !bcrypt::verify(&req.password, &user.1).unwrap_or(false) {
        return Err(AppError::InvalidCredentials);
    }

    drop(conn);

    let token = state.jwt.generate(&req.username);
    tracing::info!(username = %req.username, action = "login", "audit");

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        expires_in: 86400,
    })))
}

pub async fn logout(
    State(_state): State<AuthState>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    tracing::info!(action = "logout", "audit");
    Ok(Json(ApiResponse { code: 0, message: "ok".into(), data: None }))
}

pub async fn me(
    State(_state): State<AuthState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    Ok(Json(ApiResponse::success(serde_json::json!({"authenticated": true}))))
}
