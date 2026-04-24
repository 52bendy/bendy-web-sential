use axum::{
    extract::State,
    routing::{post, get},
    Router, Json,
};
use std::sync::Arc;
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::security::{JwtServiceClone, TokenBlacklist};
use crate::types::{ApiResponse, LoginResponse};
use crate::error::AppError;

#[derive(Clone)]
pub struct AuthState {
    pub db: DbPool,
    pub jwt: JwtServiceClone,
    pub blacklist: Arc<TokenBlacklist>,
}

pub fn router(db: DbPool, config: &AppConfig, blacklist: Arc<TokenBlacklist>) -> Router {
    let jwt = JwtServiceClone::new(config.jwt_secret.clone(), config.jwt_expiry_secs);
    let state = AuthState { db, jwt, blacklist };

    Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/me", get(me))
        .route("/api/v1/auth/totp/setup", post(setup_totp))
        .route("/api/v1/auth/totp/verify", post(verify_totp))
        .route("/api/v1/auth/totp/disable", post(disable_totp))
        .with_state(state)
}

#[derive(serde::Deserialize)]
pub struct LoginWithTotp {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub otpauth_url: String,
}

fn verify_totp_code(secret_b32: &str, code: &str) -> Result<bool, AppError> {
    let secret = totp_rs::Secret::Encoded(secret_b32.to_string());
    let secret_bytes = secret.to_bytes().map_err(|_e| AppError::InternalError)?;
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("bws".to_string()),
        "admin".to_string(),
    ).map_err(|_e| AppError::InternalError)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Ok(totp.check(code, now))
}

pub async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginWithTotp>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let user: Option<(i64, String, Option<String>)> = conn.query_row(
        "SELECT id, password_hash, totp_secret FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [&req.username],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).ok();

    let user = user.ok_or(AppError::InvalidCredentials)?;

    if !bcrypt::verify(&req.password, &user.1).unwrap_or(false) {
        return Err(AppError::InvalidCredentials);
    }

    // TOTP verification if enabled
    if let Some(ref stored) = user.2 {
        if stored.starts_with("CONFIRMED:") {
            let encrypted = stored.strip_prefix("CONFIRMED:").unwrap();
            let decrypted = crate::security::totp::decrypt_secret(encrypted)?;
            let code = req.totp_code.as_ref().ok_or(AppError::InsufficientPermissions)?;
            if !verify_totp_code(&decrypted, code)? {
                return Err(AppError::InvalidCredentials);
            }
        }
    }

    drop(conn);

    let (token, jti) = state.jwt.generate(&req.username);
    tracing::info!(username = %req.username, action = "login", "audit");

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        expires_in: 86400,
        jti: Some(jti),
    })))
}

pub async fn logout(
    State(state): State<AuthState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if let Some(jti) = payload["jti"].as_str() {
        state.blacklist.revoke(jti, std::time::Duration::from_secs(86400));
    }
    tracing::info!(action = "logout", "audit");
    Ok(Json(ApiResponse::ok()))
}

pub async fn me(
    State(_state): State<AuthState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    Ok(Json(ApiResponse::success(serde_json::json!({"authenticated": true}))))
}

pub async fn setup_totp(
    State(state): State<AuthState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<TotpSetupResponse>>, AppError> {
    let username = payload["username"].as_str().unwrap_or("");
    let password = payload["password"].as_str().unwrap_or("");

    // Verify password first
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let user: (i64, String) = conn.query_row(
        "SELECT id, password_hash FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [username],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).ok().ok_or(AppError::InvalidCredentials)?;

    if !bcrypt::verify(password, &user.1).unwrap_or(false) {
        return Err(AppError::InvalidCredentials);
    }
    drop(conn);

    // Generate TOTP secret
    let secret = totp_rs::Secret::generate_secret();
    let secret_b32 = secret.to_encoded().to_string();
    let secret_bytes = secret.to_bytes().map_err(|_e| AppError::InternalError)?;
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("bws".to_string()),
        username.to_string(),
    ).map_err(|_e| AppError::InternalError)?;

    // Store pending secret (unconfirmed)
    let encrypted = crate::security::totp::encrypt_secret(&secret_b32)?;
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "UPDATE bws_admin_users SET totp_secret = ?1 WHERE username = ?2",
        rusqlite::params![format!("PENDING:{}", encrypted), username],
    )?;

    tracing::info!(username = %username, action = "totp_setup", "audit");

    Ok(Json(ApiResponse::success(TotpSetupResponse {
        secret: secret_b32,
        otpauth_url: totp.get_url(),
    })))
}

pub async fn verify_totp(
    State(state): State<AuthState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let username = payload["username"].as_str().unwrap_or("");
    let code = payload["code"].as_str().ok_or(AppError::InvalidParam)?;

    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let pending: Option<String> = conn.query_row(
        "SELECT totp_secret FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [username],
        |row| row.get(0),
    ).ok();

    if let Some(ref secret) = pending {
        if let Some(encrypted) = secret.strip_prefix("PENDING:") {
            let decrypted = crate::security::totp::decrypt_secret(encrypted)?;
            if verify_totp_code(&decrypted, code)? {
                // Confirm the TOTP
                conn.execute(
                    "UPDATE bws_admin_users SET totp_secret = ?1 WHERE username = ?2",
                    rusqlite::params![format!("CONFIRMED:{}", encrypted), username],
                )?;
                tracing::info!(username = %username, action = "totp_confirmed", "audit");
                return Ok(Json(ApiResponse::success(serde_json::json!({"enabled": true}))));
            }
        }
    }
    Ok(Json(ApiResponse {
        code: 1003,
        message: "Invalid code".into(),
        data: None,
    }))
}

pub async fn disable_totp(
    State(state): State<AuthState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let username = payload["username"].as_str().unwrap_or("");
    let password = payload["password"].as_str().unwrap_or("");
    let code = payload["code"].as_str().ok_or(AppError::InvalidParam)?;

    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let user: (String, Option<String>) = conn.query_row(
        "SELECT password_hash, totp_secret FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [username],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).ok().ok_or(AppError::InvalidCredentials)?;

    if !bcrypt::verify(password, &user.0).unwrap_or(false) {
        return Err(AppError::InvalidCredentials);
    }

    if let Some(ref secret) = user.1 {
        let clean = secret.strip_prefix("CONFIRMED:").unwrap_or(secret);
        let decrypted = crate::security::totp::decrypt_secret(clean)?;
        if !verify_totp_code(&decrypted, code)? {
            return Err(AppError::InvalidCredentials);
        }
        conn.execute(
            "UPDATE bws_admin_users SET totp_secret = NULL WHERE username = ?1",
            [username],
        )?;
        tracing::info!(username = %username, action = "totp_disabled", "audit");
    }

    Ok(Json(ApiResponse::ok()))
}
