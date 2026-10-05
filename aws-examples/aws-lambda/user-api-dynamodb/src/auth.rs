use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
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
    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(error) => Err(AppError::Auth(format!(
            "password verification failed: {error}"
        ))),
    }
}

/// Perform Argon2 work for an unknown account to reduce the timing difference.
/// This does not guarantee constant response times; cold initialization costs extra.
pub fn verify_password_for_unknown_user(password: &str) -> Result<(), AppError> {
    static DUMMY_HASH: OnceLock<Result<String, String>> = OnceLock::new();
    let hash =
        DUMMY_HASH.get_or_init(|| hash_password("dummy-password").map_err(|e| e.to_string()));
    let hash = hash.as_ref().map_err(|e| AppError::Auth(e.clone()))?;
    verify_password(password, hash)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims<'a> {
    sub: &'a str,
    #[serde(rename = "fid")]
    family_id: &'a str,
    exp: u64,
}

/// The identity a valid access token proves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub user_id: String,
    pub family_id: String,
}

#[derive(Deserialize)]
struct OwnedClaims {
    sub: String,
    fid: String,
    exp: u64,
}

/// Check an access token: the signature must match our secret, the algorithm
/// must be HS256 (no others are accepted), and it must not have expired.
pub fn verify_jwt(secret: &str, token: &str) -> Result<Identity, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    let data = decode::<OwnedClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Auth(format!("invalid access token: {e}")))?;
    if u64::try_from(current_epoch_seconds()?)
        .map_err(|_| AppError::Auth("negative epoch".into()))?
        >= data.claims.exp
    {
        return Err(AppError::Auth("access token expired".into()));
    }
    Ok(Identity {
        user_id: data.claims.sub,
        family_id: data.claims.fid,
    })
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
        .as_secs();
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
        .map(|d| d.as_secs())
        .map_err(|e| AppError::Auth(format!("invalid system time: {e}")))
        .and_then(|seconds| {
            i64::try_from(seconds).map_err(|_| AppError::Auth("epoch overflow".into()))
        })
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
    fn issued_tokens_verify_and_carry_the_identity() {
        let token = issue_jwt("secret", "user-1", "fam-1", 60).expect("token");
        let identity = verify_jwt("secret", &token).expect("valid");
        assert_eq!(
            identity,
            Identity {
                user_id: "user-1".into(),
                family_id: "fam-1".into()
            }
        );
    }

    #[test]
    fn forged_expired_and_garbage_tokens_are_rejected() {
        let token = issue_jwt("secret", "user-1", "fam-1", 60).expect("token");
        assert!(verify_jwt("another-secret", &token).is_err()); // signed with a different key
        assert!(verify_jwt("secret", "not-a-jwt").is_err());
        // This example disables the library's default clock-skew allowance.
        let expired = encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                sub: "user-1",
                family_id: "fam-1",
                exp: 1_000,
            },
            &EncodingKey::from_secret(b"secret"),
        )
        .unwrap();
        assert!(verify_jwt("secret", &expired).is_err());
    }
    #[test]
    fn a_token_expiring_this_second_is_already_expired() {
        let token = encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                sub: "u",
                family_id: "f",
                exp: current_epoch_seconds().unwrap() as u64,
            },
            &EncodingKey::from_secret(b"secret"),
        )
        .unwrap();
        assert!(verify_jwt("secret", &token).is_err());
    }
}
