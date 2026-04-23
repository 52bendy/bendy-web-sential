use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtPayload {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct JwtServiceClone {
    pub secret: String,
    pub expiry_secs: u64,
}

impl JwtServiceClone {
    pub fn new(secret: String, expiry_secs: u64) -> Self {
        Self { secret, expiry_secs }
    }

    pub fn generate(&self, username: &str) -> String {
        let now = Utc::now();
        let exp = (now + Duration::seconds(self.expiry_secs as i64)).timestamp() as usize;
        let payload = JwtPayload {
            sub: username.to_string(),
            exp,
            iat: now.timestamp() as usize,
        };
        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .expect("jwt encode")
    }

    pub fn verify(&self, token: &str) -> Result<JwtPayload, jsonwebtoken::errors::Error> {
        let validation = Validation::default();
        let token_data = decode::<JwtPayload>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;
        Ok(token_data.claims)
    }
}
