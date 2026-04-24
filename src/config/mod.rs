use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub gateway_port: u16,
    pub admin_port: u16,
    pub jwt_secret: String,
    pub jwt_expiry_secs: u64,
    pub database_url: String,
    pub log_level: String,
    pub totp_aes_key: Option<String>,
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
        }
    }
}
