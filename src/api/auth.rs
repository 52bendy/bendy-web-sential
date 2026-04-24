use axum::{
    extract::State,
    routing::{post, get, put},
    Router,
    Json,
    body::Body,
    http::Request,
    http::header::AUTHORIZATION,
    http::request::Parts,
};
use serde::Deserialize;
use std::sync::Arc;

// Custom extractor that gets auth token from headers without consuming body
pub struct AuthToken(pub String);

impl<S> axum::extract::FromRequestParts<S> for AuthToken
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .ok_or(AppError::AuthRequired)?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .unwrap_or(auth_header)
                .trim()
                .to_owned();

            Ok(AuthToken(token))
        }
    }
}
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
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub github_redirect_uri: Option<String>,
    pub github_admin_emails: Vec<String>,
    pub frontend_url: String,
    pub sso_enabled: bool,
    pub sso_secret: Option<String>,
}

pub fn router(db: DbPool, config: &AppConfig, blacklist: Arc<TokenBlacklist>) -> Router {
    let jwt = JwtServiceClone::new(config.jwt_secret.clone(), config.jwt_expiry_secs);
    let state = AuthState {
        db,
        jwt: jwt.clone(),
        blacklist,
        github_client_id: config.github_client_id.clone(),
        github_client_secret: config.github_client_secret.clone(),
        github_redirect_uri: config.github_redirect_uri.clone(),
        github_admin_emails: config.github_admin_emails.clone(),
        frontend_url: config.frontend_url.clone(),
        sso_enabled: config.sso_enabled,
        sso_secret: config.sso_secret.clone(),
    };

    Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/me", get(me))
        .route("/api/v1/auth/me", put(update_profile))
        .route("/api/v1/auth/totp/setup", post(setup_totp))
        .route("/api/v1/auth/totp/verify", post(verify_totp))
        .route("/api/v1/auth/totp/disable", post(disable_totp))
        .route("/api/v1/auth/sso/exchange", post(sso_exchange))
        .route("/api/auth/github/login", get(github_login))
        .route("/api/auth/github/callback", get(github_callback))
        .with_state(state)
}

fn extract_bearer_token(req: &Request<Body>) -> Option<String> {
    req.headers()
        .get("authorization")?
        .to_str().ok()?
        .strip_prefix("Bearer ")?
        .trim()
        .to_owned()
        .into()
}

fn authenticated_username(req: &http::request::Parts, jwt: &JwtServiceClone) -> Result<String, AppError> {
    let token = extract_bearer_token(&Request::from_parts(req.clone(), Body::default())).ok_or(AppError::AuthRequired)?;
    let claims = jwt.verify(&token).map_err(|_| AppError::TokenInvalid)?;
    Ok(claims.sub)
}

#[derive(serde::Deserialize)]
pub struct LoginWithTotp {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(serde::Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub avatar: Option<String>,
    pub role: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub avatar: Option<String>,
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
    State(state): State<AuthState>,
    auth: AuthToken,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let claims = state.jwt.verify(&auth.0).map_err(|_| AppError::TokenInvalid)?;
    let username = claims.sub;
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let user: (i64, String, Option<String>, String) = conn.query_row(
        "SELECT id, username, avatar, role FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [&username],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|_| AppError::NotFound)?;
    Ok(Json(ApiResponse::success(UserInfo {
        id: user.0,
        username: user.1,
        avatar: user.2,
        role: user.3,
    })))
}

#[axum::debug_handler]
pub async fn update_profile(
    State(state): State<AuthState>,
    auth: AuthToken,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let claims = state.jwt.verify(&auth.0).map_err(|_| AppError::TokenInvalid)?;
    let current_username = claims.sub;
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;

    let effective_username = if let Some(ref new_name) = payload.username {
        if !new_name.is_empty() {
            conn.execute(
                "UPDATE bws_admin_users SET username = ?1, updated_at = ?2 WHERE username = ?3",
                rusqlite::params![new_name, chrono::Utc::now().to_rfc3339(), &current_username],
            )?;
            new_name.clone()
        } else {
            current_username.clone()
        }
    } else {
        current_username.clone()
    };

    if let Some(ref avatar_url) = payload.avatar {
        conn.execute(
            "UPDATE bws_admin_users SET avatar = ?1, updated_at = ?2 WHERE username = ?3",
            rusqlite::params![avatar_url, chrono::Utc::now().to_rfc3339(), &effective_username],
        )?;
    }

    let user: (i64, String, Option<String>, String) = conn.query_row(
        "SELECT id, username, avatar, role FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [&effective_username],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|_| AppError::NotFound)?;

    Ok(Json(ApiResponse::success(UserInfo {
        id: user.0,
        username: user.1,
        avatar: user.2,
        role: user.3,
    })))
}

#[axum::debug_handler]
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

// GitHub OAuth

/// GET /api/v1/auth/github/login
/// Redirects to GitHub OAuth authorization page
#[axum::debug_handler]
pub async fn github_login(
    State(state): State<AuthState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let client_id = state.github_client_id.as_ref().ok_or(AppError::ConfigError("GitHub OAuth not configured".into()))?;
    let redirect_uri = state.github_redirect_uri.as_ref().ok_or(AppError::ConfigError("GitHub OAuth not configured".into()))?;

    let state_param = uuid::Uuid::new_v4().to_string();
    let params = [
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("scope", "user:email"),
        ("state", &state_param),
    ];
    let query_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let auth_url = format!("https://github.com/login/oauth/authorize?{}", query_string);
    Ok(axum::response::Redirect::to(&auth_url))
}

#[derive(Debug, Deserialize)]
pub struct GithubCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubTokenResponse {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
}

/// GET /api/auth/github/callback
/// Exchanges code for token and logs in user
pub async fn github_callback(
    State(state): State<AuthState>,
    axum::extract::Query(query): axum::extract::Query<GithubCallbackQuery>,
) -> Result<axum::response::Redirect, AppError> {
    tracing::info!("github callback called, error: {:?}", query.error);
    if let Some(err) = &query.error {
        tracing::warn!("github oauth error: {}", err);
        return Err(AppError::InvalidCredentials);
    }

    let code = query.code.as_ref().ok_or(AppError::InvalidCredentials)?;
    let client_id = state.github_client_id.as_ref().ok_or(AppError::ConfigError("GitHub OAuth not configured".into()))?;
    let client_secret = state.github_client_secret.as_ref().ok_or(AppError::ConfigError("GitHub OAuth not configured".into()))?;
    let redirect_uri = state.github_redirect_uri.as_ref().ok_or(AppError::ConfigError("GitHub OAuth not configured".into()))?;

    // Exchange code for access token
    tracing::info!("exchanging code for access token, redirect_uri: {}", redirect_uri);
    let token_res: GithubTokenResponse = reqwest::Client::new()
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
            "redirect_uri": redirect_uri,
        }))
        .send()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?
        .json::<GithubTokenResponse>()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?;

    tracing::info!("github token response: {:?}", token_res);

    // Check for GitHub OAuth errors
    if let Some(err) = &token_res.error {
        let desc = token_res.error_description.as_deref().unwrap_or("");
        tracing::error!("github oauth error: {} - {}", err, desc);
        return Err(AppError::ConfigError(format!("GitHub OAuth failed: {} - {}", err, desc)));
    }

    let access_token = token_res.access_token.as_ref().ok_or_else(|| {
        tracing::error!("no access_token in github response");
        AppError::InvalidCredentials
    })?;

    // Get user info from GitHub
    let github_user: GithubUser = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "bendy-web-sential")
        .send()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?
        .json()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?;

    // Get emails from GitHub
    #[derive(Debug, Deserialize)]
    struct GithubEmail {
        email: String,
        primary: bool,
        verified: bool,
    }
    let emails: Vec<GithubEmail> = reqwest::Client::new()
        .get("https://api.github.com/user/emails")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "bendy-web-sential")
        .send()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?
        .json()
        .await
        .map_err(|e| AppError::UpstreamError(e.to_string()))?;

    // Check if user email matches any admin email from config
    let is_admin = state.github_admin_emails.iter()
        .any(|admin_email| emails.iter()
            .filter(|e| e.verified && e.primary)
            .any(|e| e.email.eq_ignore_ascii_case(admin_email)));

    let user_email = emails.iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email.clone())
        .or(github_user.email.clone());

    let username = format!("gh_{}", github_user.login);
    let avatar = github_user.avatar_url;
    let now = chrono::Utc::now().to_rfc3339();
    let role = if is_admin { "admin" } else { "user" };

    // Upsert user in database
    {
        let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
        let exists: bool = conn.query_row(
            "SELECT 1 FROM bws_admin_users WHERE username = ?1",
            [&username],
            |_| Ok(true),
        ).unwrap_or(false);

        if exists {
            conn.execute(
                "UPDATE bws_admin_users SET avatar = ?1, email = COALESCE(?2, email), updated_at = ?3 WHERE username = ?4",
                rusqlite::params![avatar, user_email, now, &username],
            )?;
        } else {
            conn.execute(
                "INSERT INTO bws_admin_users (username, password_hash, avatar, email, role, active, created_at, updated_at)
                 VALUES (?1, '', ?2, ?3, ?4, 1, ?5, ?5)",
                rusqlite::params![&username, avatar, user_email, role, now],
            )?;
        }
    }

    tracing::info!(username = %username, role = %role, action = "github_login", "audit");

    let (token, jti) = state.jwt.generate(&username);

    let redirect_url = format!("{}/login?token={}&jti={}", state.frontend_url.trim_end_matches('/'), token, jti);
    Ok(axum::response::Redirect::to(&redirect_url))
}

#[derive(Debug, Deserialize)]
pub struct SsoExchangeRequest {
    pub token: String,
}

/// POST /api/v1/auth/sso/exchange
/// Exchange an SSO token for a session JWT
pub async fn sso_exchange(
    State(state): State<AuthState>,
    Json(payload): Json<SsoExchangeRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    if !state.sso_enabled {
        return Err(AppError::ConfigError("SSO token login is not enabled".into()));
    }

    let secret = state.sso_secret.as_ref().ok_or(AppError::ConfigError("SSO secret not configured".into()))?;

    // Parse and validate the SSO token
    let parts: Vec<&str> = payload.token.split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::TokenInvalid);
    }

    // Verify signature (HS256)
    let _header = base64_url_decode(parts[0])?;
    let payload_bytes = base64_url_decode(parts[1])?;
    let signature_base = format!("{}.{}", parts[0], parts[1]);

    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::InternalError)?;
    mac.update(signature_base.as_bytes());
    let expected = mac.finalize().into_bytes();
    let provided = base64_url_decode(parts[2])?;

    if expected.as_slice() != provided.as_slice() {
        return Err(AppError::TokenInvalid);
    }

    #[derive(serde::Deserialize)]
    struct SsoClaims {
        sub: String,
        exp: usize,
        iat: usize,
        #[serde(default)]
        role: Option<String>,
    }

    let claims: SsoClaims = serde_json::from_slice(&payload_bytes)
        .map_err(|_| AppError::TokenInvalid)?;

    // Validate expiration
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    if claims.exp < now {
        return Err(AppError::TokenExpired);
    }

    // Verify user exists in database
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    let user_exists: bool = conn.query_row(
        "SELECT 1 FROM bws_admin_users WHERE username = ?1 AND active = 1",
        [&claims.sub],
        |_| Ok(true),
    ).unwrap_or(false);
    drop(conn);

    if !user_exists {
        return Err(AppError::InvalidCredentials);
    }

    tracing::info!(username = %claims.sub, action = "sso_exchange", "audit");

    let (jwt, jti) = state.jwt.generate(&claims.sub);
    Ok(Json(ApiResponse::success(LoginResponse {
        token: jwt,
        expires_in: 86400,
        jti: Some(jti),
    })))
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, AppError> {
    // Replace URL-safe chars with standard base64 chars
    let normalized = input.replace('-', "+").replace('_', "/");
    // Add padding if needed
    let padded = match normalized.len() % 4 {
        2 => format!("{}==", normalized),
        3 => format!("{}=", normalized),
        _ => normalized,
    };
    // Use base64 crate to decode
    let bytes = base64::decode(&padded).map_err(|_| AppError::TokenInvalid)?;
    Ok(bytes)
}
