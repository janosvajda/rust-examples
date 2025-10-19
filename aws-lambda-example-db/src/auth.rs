use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// Default TTL for access tokens (15 minutes).
pub const ACCESS_TOKEN_TTL_SECONDS: u64 = 15 * 60;
/// Default TTL for refresh tokens (7 days).
pub const REFRESH_TOKEN_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// Hash a plaintext password using Argon2id.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|ph| ph.to_string())
        .map_err(|e| AppError::Auth(format!("failed to hash password: {e}")))
}

/// Verify a plaintext password against a stored hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Auth(format!("invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims<'a> {
    sub: &'a str,
    #[serde(rename = "fid")]
    family_id: &'a str,
    exp: usize,
}

/// Issue a JWT access token for the provided principal.
pub fn issue_jwt(
    secret: &str,
    user_id: &str,
    family_id: &str,
    ttl_seconds: u64,
) -> Result<String, AppError> {
    let header = Header::new(Algorithm::HS256);
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(ttl_seconds))
        .ok_or_else(|| AppError::Auth("expiration overflow".to_string()))?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Auth(format!("invalid system time: {e}")))?
        .as_secs() as usize;
    let claims = Claims {
        sub: user_id,
        family_id,
        exp: expiration,
    };
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Auth(format!("failed to sign JWT: {e}")))
}

/// Generate a random opaque refresh token.
pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

/// Current UNIX epoch seconds.
pub fn current_epoch_seconds() -> Result<i64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|e| AppError::Auth(format!("invalid system time: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_and_verify() {
        let hash = hash_password("super-secret").expect("hash");
        assert!(verify_password("super-secret", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn issues_jwt() {
        let token = issue_jwt("secret", "user-1", "fam-1", 60).expect("token");
        assert!(!token.is_empty());
    }
}
