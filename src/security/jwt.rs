use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtPayload {
    pub sub: String,
    pub jti: String,
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

    pub fn generate(&self, username: &str) -> (String, String) {
        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();
        let exp = (now + Duration::seconds(self.expiry_secs as i64)).timestamp() as usize;
        let payload = JwtPayload {
            sub: username.to_string(),
            jti: jti.clone(),
            exp,
            iat: now.timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .expect("jwt encode");
        (token, jti)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generate_and_verify() {
        let service = JwtServiceClone::new("test-secret-key-at-least-32-bytes-long".to_string(), 3600);
        let (token, jti) = service.generate("admin");

        assert!(!token.is_empty());
        assert!(!jti.is_empty());

        let payload = service.verify(&token).unwrap();
        assert_eq!(payload.sub, "admin");
        assert_eq!(payload.jti, jti);
    }

    #[test]
    fn test_jwt_verify_invalid_token() {
        let service = JwtServiceClone::new("test-secret-key-at-least-32-bytes-long".to_string(), 3600);
        let result = service.verify("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_generates_unique_tokens() {
        let service = JwtServiceClone::new("test-secret-key-at-least-32-bytes-long".to_string(), 3600);
        let (token1, jti1) = service.generate("admin");
        let (token2, jti2) = service.generate("admin");

        assert_ne!(token1, token2, "tokens should be unique");
        assert_ne!(jti1, jti2, "jtis should be unique");
    }
}
