//! Authentication middleware for gateway routes
//! Handles JWT validation and API Key validation based on route auth_strategy

use sha2::{Sha256, Digest};
use axum::http::{header::AUTHORIZATION, header::HeaderMap};
use crate::db::DbPool;
use crate::security::JwtServiceClone;
use crate::types::AuthStrategy;
use crate::error::AppError;

/// Auth result containing user info if authenticated
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub username: Option<String>,
    pub role: Option<String>,
    pub api_key_name: Option<String>,
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string())
}

/// Extract API key from X-API-Key header
pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
}

/// Check authentication for a request based on the route's auth strategy
/// Takes pre-extracted credentials to avoid borrowing Request across await
pub async fn check_route_auth(
    strategy: &AuthStrategy,
    min_role: &Option<String>,
    jwt_service: &JwtServiceClone,
    db: &DbPool,
    bearer_token: Option<String>,
    api_key: Option<String>,
) -> Result<AuthResult, AppError> {
    match strategy {
        AuthStrategy::None => {
            Ok(AuthResult {
                username: None,
                role: None,
                api_key_name: None,
            })
        }
        AuthStrategy::Jwt => {
            check_jwt_auth(min_role, jwt_service, bearer_token).await
        }
        AuthStrategy::ApiKey => {
            check_api_key_auth(min_role, db, api_key.as_deref()).await
        }
    }
}

/// Check JWT authentication from bearer token
async fn check_jwt_auth(
    min_role: &Option<String>,
    jwt_service: &JwtServiceClone,
    bearer_token: Option<String>,
) -> Result<AuthResult, AppError> {
    let token = bearer_token.ok_or(AppError::AuthRequired)?;

    // Verify JWT token
    let claims = jwt_service.verify(&token).map_err(|e| {
        tracing::warn!(error = %e, "jwt verification failed");
        AppError::TokenInvalid
    })?;

    // Get username from token
    let username = claims.sub;

    // Check role requirement
    if let Some(required_role) = min_role {
        let user_role = get_user_role(&username)?;
        if !role_meets(&user_role, required_role) {
            tracing::warn!(username = %username, required_role = %required_role, user_role = %user_role, "role requirement not met");
            return Err(AppError::InsufficientPermissions);
        }
    }

    Ok(AuthResult {
        username: Some(username),
        role: None,
        api_key_name: None,
    })
}

/// Check API Key authentication
async fn check_api_key_auth(
    min_role: &Option<String>,
    db: &DbPool,
    api_key: Option<&str>,
) -> Result<AuthResult, AppError> {
    let key = api_key.ok_or(AppError::AuthRequired)?;

    // Hash the API key
    let key_hash = hash_api_key(key);

    let conn = db.lock().map_err(|_| AppError::InternalError)?;

    // Look up API key
    let result: Option<(String, String, Option<String>, Option<String>)> = conn.query_row(
        "SELECT name, role, expires_at, last_used_at FROM bws_api_keys WHERE key_hash = ?1 AND active = 1",
        [&key_hash],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).ok();

    let result = result.ok_or(AppError::InvalidCredentials)?;
    let (name, role, expires_at, _last_used_at) = result;

    // Check expiration
    if let Some(expires) = expires_at {
        let expiry = chrono::DateTime::parse_from_rfc3339(&expires)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .ok();
        if let Some(expiry) = expiry {
            if expiry < chrono::Utc::now() {
                tracing::warn!(key_name = %name, "api key expired");
                return Err(AppError::TokenExpired);
            }
        }
    }

    // Check role requirement
    if let Some(required_role) = min_role {
        if !role_meets(&role, required_role) {
            tracing::warn!(key_name = %name, required_role = %required_role, key_role = %role, "role requirement not met");
            return Err(AppError::InsufficientPermissions);
        }
    }

    // Update last used timestamp
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE bws_api_keys SET last_used_at = ?1 WHERE key_hash = ?2",
        rusqlite::params![now, key_hash],
    );

    Ok(AuthResult {
        username: None,
        role: Some(role),
        api_key_name: Some(name),
    })
}

/// Hash API key using SHA256
fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::default();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Get user role from database
fn get_user_role(_username: &str) -> Result<String, AppError> {
    // This is a simplified version - in practice would use DB connection from state
    // For now, assume the caller has access to the DB
    Ok("user".to_string())
}

/// Check if a role meets the minimum requirement
/// Role hierarchy: superadmin > admin > user
pub fn role_meets(user_role: &str, required_role: &str) -> bool {
    let user_level = role_level(user_role);
    let required_level = role_level(required_role);
    user_level >= required_level
}

fn role_level(role: &str) -> u8 {
    match role.to_lowercase().as_str() {
        "superadmin" => 3,
        "admin" => 2,
        "user" => 1,
        _ => 0,
    }
}

// =============================================================================
// API Key Management
// =============================================================================

use rusqlite::params;

/// Create a new API key
pub fn create_api_key(
    db: &DbPool,
    name: &str,
    role: &str,
    expires_at: Option<&str>,
) -> Result<(String, String), AppError> {
    let conn = db.lock().map_err(|_| AppError::InternalError)?;

    // Generate a random API key
    let raw_key = generate_api_key();
    let key_hash = hash_api_key(&raw_key);
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO bws_api_keys (key_hash, name, role, active, created_at, expires_at)
         VALUES (?1, ?2, ?3, 1, ?4, ?5)",
        params![key_hash, name, role, now, expires_at],
    )?;

    Ok((raw_key, key_hash))
}

/// Generate a random API key
fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Revoke an API key
pub fn revoke_api_key(db: &DbPool, key_hash: &str) -> Result<(), AppError> {
    let conn = db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute(
        "UPDATE bws_api_keys SET active = 0 WHERE key_hash = ?1",
        [key_hash],
    )?;
    Ok(())
}

/// List all API keys (without revealing the hash)
pub fn list_api_keys(db: &DbPool) -> Result<Vec<ApiKeyInfo>, AppError> {
    let conn = db.lock().map_err(|_| AppError::InternalError)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, role, active, created_at, expires_at, last_used_at FROM bws_api_keys ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        let expires_at: Option<String> = row.get(5)?;
        let last_used_at: Option<String> = row.get(6)?;

        Ok(ApiKeyInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            role: row.get(2)?,
            active: row.get::<_, i32>(3)? == 1,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .unwrap().with_timezone(&chrono::Utc),
            expires_at: expires_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s).ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            last_used_at: last_used_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s).ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
        })
    })?;

    let keys: Vec<_> = rows.filter_map(|r| r.ok()).collect();
    Ok(keys)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiKeyInfo {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_meets() {
        // Superadmin can access everything
        assert!(role_meets("superadmin", "user"));
        assert!(role_meets("superadmin", "admin"));
        assert!(role_meets("superadmin", "superadmin"));

        // Admin can access admin and user
        assert!(role_meets("admin", "user"));
        assert!(role_meets("admin", "admin"));
        assert!(!role_meets("admin", "superadmin"));

        // User can only access user
        assert!(role_meets("user", "user"));
        assert!(!role_meets("user", "admin"));
        assert!(!role_meets("user", "superadmin"));
    }

    #[test]
    fn test_hash_api_key() {
        let key = "test-api-key-123";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
    }
}
