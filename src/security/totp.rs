use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

const NONCE_SIZE: usize = 12;

pub fn encrypt_secret(secret: &str) -> Result<String, crate::error::AppError> {
    let key = get_aes_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| crate::error::AppError::ConfigError(e.to_string()))?;
    let nonce_bytes: [u8; NONCE_SIZE] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, secret.as_bytes())
        .map_err(|e| crate::error::AppError::ConfigError(e.to_string()))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);
    Ok(base64::encode(&combined))
}

pub fn decrypt_secret(encrypted: &str) -> Result<String, crate::error::AppError> {
    let key = get_aes_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| crate::error::AppError::ConfigError(e.to_string()))?;
    let data = base64::decode(encrypted)
        .map_err(|_| crate::error::AppError::ConfigError("base64 decode failed".into()))?;

    if data.len() < NONCE_SIZE {
        return Err(crate::error::AppError::ConfigError("data too short".into()));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| crate::error::AppError::ConfigError("decryption failed".into()))?;

    String::from_utf8(plaintext)
        .map_err(|_| crate::error::AppError::ConfigError("utf8 decode failed".into()))
}

fn get_aes_key() -> Result<[u8; 32], crate::error::AppError> {
    let key_str = std::env::var("BWS_TOTP_AES_KEY")
        .or_else(|_| std::env::var("BWS_AES_KEY"))
        .map_err(|_| crate::error::AppError::ConfigError("BWS_TOTP_AES_KEY not set".into()))?;

    let key_bytes = base64::decode(&key_str)
        .map_err(|e| crate::error::AppError::ConfigError(format!("key decode error: {}", e)))?;

    let mut key = [0u8; 32];
    if key_bytes.len() < 32 {
        key[..key_bytes.len()].copy_from_slice(&key_bytes);
    } else {
        key.copy_from_slice(&key_bytes[..32]);
    }
    Ok(key)
}
