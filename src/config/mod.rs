use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub ip_enabled: bool,
    pub ip_limit_per_second: u64,
    pub global_enabled: bool,
    pub global_limit_per_second: u64,
    pub path_enabled: bool,
    pub path_limit_per_second: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            ip_enabled: true,
            ip_limit_per_second: 10,
            global_enabled: true,
            global_limit_per_second: 1000,
            path_enabled: false,
            path_limit_per_second: 100,
        }
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            ip_enabled: env::var("BWS_RATE_LIMIT_IP_ENABLED")
                .unwrap_or_else(|_| "true".into())
                .parse()
                .unwrap_or(true),
            ip_limit_per_second: env::var("BWS_RATE_LIMIT_IP_PER_SECOND")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            global_enabled: env::var("BWS_RATE_LIMIT_GLOBAL_ENABLED")
                .unwrap_or_else(|_| "true".into())
                .parse()
                .unwrap_or(true),
            global_limit_per_second: env::var("BWS_RATE_LIMIT_GLOBAL_PER_SECOND")
                .unwrap_or_else(|_| "1000".into())
                .parse()
                .unwrap_or(1000),
            path_enabled: env::var("BWS_RATE_LIMIT_PATH_ENABLED")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            path_limit_per_second: env::var("BWS_RATE_LIMIT_PATH_PER_SECOND")
                .unwrap_or_else(|_| "100".into())
                .parse()
                .unwrap_or(100),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub half_open_max_requests: u32,
    pub open_timeout_secs: u64,
    pub request_volume_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            failure_threshold: 5,
            success_threshold: 3,
            half_open_max_requests: 3,
            open_timeout_secs: 30,
            request_volume_threshold: 10,
        }
    }
}

impl CircuitBreakerConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            enabled: env::var("BWS_CIRCUIT_BREAKER_ENABLED")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            failure_threshold: env::var("BWS_CIRCUIT_BREAKER_FAILURE_THRESHOLD")
                .unwrap_or_else(|_| "5".into())
                .parse()
                .unwrap_or(5),
            success_threshold: env::var("BWS_CIRCUIT_BREAKER_SUCCESS_THRESHOLD")
                .unwrap_or_else(|_| "3".into())
                .parse()
                .unwrap_or(3),
            half_open_max_requests: env::var("BWS_CIRCUIT_BREAKER_HALF_OPEN_MAX")
                .unwrap_or_else(|_| "3".into())
                .parse()
                .unwrap_or(3),
            open_timeout_secs: env::var("BWS_CIRCUIT_BREAKER_OPEN_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
            request_volume_threshold: env::var("BWS_CIRCUIT_BREAKER_VOLUME_THRESHOLD")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    pub enabled: bool,
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

impl RetryConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            enabled: env::var("BWS_RETRY_ENABLED")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            max_attempts: env::var("BWS_RETRY_MAX_ATTEMPTS")
                .unwrap_or_else(|_| "3".into())
                .parse()
                .unwrap_or(3),
            base_delay_ms: env::var("BWS_RETRY_BASE_DELAY_MS")
                .unwrap_or_else(|_| "100".into())
                .parse()
                .unwrap_or(100),
            max_delay_ms: env::var("BWS_RETRY_MAX_DELAY_MS")
                .unwrap_or_else(|_| "5000".into())
                .parse()
                .unwrap_or(5000),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub dir: String,
    pub interval_hours: u32,
    pub retention_days: u32,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dir: "backups".into(),
            interval_hours: 24,
            retention_days: 7,
        }
    }
}

impl BackupConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("BWS_BACKUP_ENABLED")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            dir: env::var("BWS_BACKUP_DIR").unwrap_or_else(|_| "backups".into()),
            interval_hours: env::var("BWS_BACKUP_INTERVAL_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()
                .unwrap_or(24),
            retention_days: env::var("BWS_BACKUP_RETENTION_DAYS")
                .unwrap_or_else(|_| "7".into())
                .parse()
                .unwrap_or(7),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub gateway_port: u16,
    pub admin_port: u16,
    pub jwt_secret: String,
    pub jwt_expiry_secs: u64,
    pub database_url: String,
    pub log_level: String,
    pub totp_aes_key: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub github_redirect_uri: Option<String>,
    pub github_admin_emails: Vec<String>,
    pub frontend_url: String,
    pub cloudflare_api_token: Option<String>,
    pub cloudflare_zone_identifier: Option<String>,
    pub rate_limit: RateLimitConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub retry: RetryConfig,
    pub backup: BackupConfig,
    pub sso_enabled: bool,
    pub sso_secret: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gateway_port: 8080,
            admin_port: 3000,
            jwt_secret: "changeme-in-env".into(),
            jwt_expiry_secs: 86400,
            database_url: "data/bws.db".into(),
            log_level: "info".into(),
            totp_aes_key: None,
            github_client_id: None,
            github_client_secret: None,
            github_redirect_uri: None,
            github_admin_emails: vec![],
            frontend_url: "http://128.140.80.71".into(),
            cloudflare_api_token: None,
            cloudflare_zone_identifier: None,
            rate_limit: RateLimitConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            retry: RetryConfig::default(),
            backup: BackupConfig::default(),
            sso_enabled: false,
            sso_secret: None,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            gateway_port: env::var("BWS_GATEWAY_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .unwrap_or(8080),
            admin_port: env::var("BWS_ADMIN_PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .unwrap_or(3000),
            jwt_secret: env::var("BWS_JWT_SECRET").unwrap_or_else(|_| "changeme-in-env".into()),
            jwt_expiry_secs: env::var("BWS_JWT_EXPIRY_SECS")
                .unwrap_or_else(|_| "86400".into())
                .parse()
                .unwrap_or(86400),
            database_url: env::var("BWS_DATABASE_URL").unwrap_or_else(|_| "data/bws.db".into()),
            log_level: env::var("BWS_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            totp_aes_key: env::var("BWS_TOTP_AES_KEY").ok(),
            github_client_id: env::var("BWS_GITHUB_CLIENT_ID").ok(),
            github_client_secret: env::var("BWS_GITHUB_CLIENT_SECRET").ok(),
            github_redirect_uri: env::var("BWS_GITHUB_REDIRECT_URI").ok(),
            github_admin_emails: env::var("BWS_GITHUB_ADMIN_EMAILS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
            frontend_url: env::var("BWS_FRONTEND_URL")
                .unwrap_or_else(|_| "http://128.140.80.71".into()),
            cloudflare_api_token: env::var("BWS_CLOUDFLARE_API_TOKEN").ok(),
            cloudflare_zone_identifier: env::var("BWS_CLOUDFLARE_ZONE_ID").ok(),
            rate_limit: RateLimitConfig::from_env(),
            circuit_breaker: CircuitBreakerConfig::from_env(),
            retry: RetryConfig::from_env(),
            backup: BackupConfig::from_env(),
            sso_enabled: env::var("BWS_SSO_ENABLED")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            sso_secret: env::var("BWS_SSO_SECRET").ok(),
        }
    }
}
