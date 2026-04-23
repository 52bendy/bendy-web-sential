use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Token expired or invalid")]
    TokenInvalid,
    #[error("Token expired")]
    TokenExpired,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Authentication required")]
    AuthRequired,
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
    #[error("Resource not found")]
    NotFound,
    #[error("Resource already exists")]
    AlreadyExists,
    #[error("Invalid parameter")]
    InvalidParam,
    #[error("Internal server error")]
    InternalError,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Upstream error: {0}")]
    UpstreamError(String),
    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::TokenInvalid | AppError::TokenExpired | AppError::AuthRequired => StatusCode::UNAUTHORIZED,
            AppError::InsufficientPermissions => StatusCode::FORBIDDEN,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::RateLimited | AppError::CircuitBreakerOpen => StatusCode::TOO_MANY_REQUESTS,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::AlreadyExists => StatusCode::CONFLICT,
            AppError::InvalidParam => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({
            "code": self.error_code(),
            "message": self.to_string(),
            "data": null
        });

        (status, Json(body)).into_response()
    }
}

impl AppError {
    pub fn error_code(&self) -> u32 {
        match self {
            AppError::TokenInvalid | AppError::TokenExpired => 1001,
            AppError::InsufficientPermissions => 1002,
            AppError::InvalidCredentials => 1003,
            AppError::AuthRequired => 1004,
            AppError::RateLimited => 2001,
            AppError::CircuitBreakerOpen => 2002,
            AppError::NotFound => 3001,
            AppError::AlreadyExists => 3002,
            AppError::InvalidParam => 1003,
            AppError::InternalError => 4001,
            AppError::DatabaseError(_) => 4002,
            AppError::ConfigError(_) => 4003,
            AppError::UpstreamError(_) => 4004,
            AppError::ServiceUnavailable => 4005,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        tracing::error!("database error: {}", e);
        AppError::DatabaseError(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        tracing::error!("jwt error: {}", e);
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::TokenInvalid,
        }
    }
}

impl From<std::env::VarError> for AppError {
    fn from(e: std::env::VarError) -> Self {
        AppError::ConfigError(e.to_string())
    }
}
