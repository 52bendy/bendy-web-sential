use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// =============================================================================
// Auth Strategy Types
// =============================================================================

/// Authentication strategy for routes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthStrategy {
    None,
    Jwt,
    ApiKey,
}

impl Default for AuthStrategy {
    fn default() -> Self {
        Self::None
    }
}

impl AuthStrategy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "jwt" => AuthStrategy::Jwt,
            "api_key" | "apikey" => AuthStrategy::ApiKey,
            _ => AuthStrategy::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthStrategy::None => "none",
            AuthStrategy::Jwt => "jwt",
            AuthStrategy::ApiKey => "api_key",
        }
    }
}

/// Rate limit dimension
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitDimension {
    Ip,
    Key,
    Global,
}

impl Default for RateLimitDimension {
    fn default() -> Self {
        Self::Ip
    }
}

impl RateLimitDimension {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "key" => RateLimitDimension::Key,
            "global" => RateLimitDimension::Global,
            _ => RateLimitDimension::Ip,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RateLimitDimension::Ip => "ip",
            RateLimitDimension::Key => "key",
            RateLimitDimension::Global => "global",
        }
    }
}

// =============================================================================
// Domain
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: i64,
    pub domain: String,
    pub description: Option<String>,
    pub hosting_service: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub id: i64,
    pub domain_id: i64,
    pub path_pattern: String,
    pub action: RouteAction,
    pub target: String,
    pub description: Option<String>,
    pub priority: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Auth fields
    pub auth_strategy: AuthStrategy,
    pub min_role: Option<String>,
    // Rate limit fields
    pub ratelimit_window: Option<i32>,
    pub ratelimit_limit: Option<i32>,
    pub ratelimit_dimension: RateLimitDimension,
    // Health check fields
    pub health_check_path: Option<String>,
    pub health_check_interval_secs: i32,
    // Transform rules (JSON string)
    pub transform_rules: Option<String>,
}

// Extended route info for gateway matching
#[derive(Debug, Clone)]
pub struct RouteWithAuth {
    pub id: i64,
    pub action: String,
    pub target: String,
    pub auth_strategy: AuthStrategy,
    pub min_role: Option<String>,
    pub ratelimit_window: Option<i32>,
    pub ratelimit_limit: Option<i32>,
    pub ratelimit_dimension: RateLimitDimension,
}

// =============================================================================
// API Key
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i64,
    pub key_hash: String,
    pub name: String,
    pub role: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Upstream (for load balancing)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upstream {
    pub id: i64,
    pub route_id: i64,
    pub target_url: String,
    pub weight: i32,
    pub active: bool,
    pub healthy: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RouteAction {
    Proxy,
    Redirect,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub totp_secret: Option<String>,
    pub role: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T = ()> {
    pub code: u32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { code: 0, message: "ok".into(), data: Some(data) }
    }

    pub fn success_msg(msg: impl Into<String>) -> ApiResponse<()> {
        ApiResponse { code: 0, message: msg.into(), data: None }
    }

    pub fn error(code: u32, message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse { code, message: message.into(), data: None }
    }
}

impl ApiResponse<()> {
    pub fn ok() -> Self {
        Self { code: 0, message: "ok".into(), data: None }
    }
}

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}
