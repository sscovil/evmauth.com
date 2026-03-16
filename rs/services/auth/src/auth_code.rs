use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Data stored alongside the hashed auth code in Redis.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthCodeData {
    pub app_registration_id: Uuid,
    pub person_id: Uuid,
    pub code_challenge: String,
    pub redirect_uri: String,
    pub client_id: String,
}

/// Generate a cryptographically random 32-byte auth code, base64url-encoded.
pub fn generate_code() -> String {
    let bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Compute SHA-256 of a plaintext code and return as hex string.
pub fn hash_code(plaintext: &str) -> String {
    let digest = Sha256::digest(plaintext.as_bytes());
    hex::encode(digest)
}

/// Compute SHA-256 of a PKCE code_verifier and return as base64url (for S256 comparison).
pub fn pkce_s256(code_verifier: &str) -> String {
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn redis_key(code_hash: &str) -> String {
    format!("auth_code:{code_hash}")
}

/// Store an auth code in Redis with TTL.
/// The key is `auth_code:{sha256_hex(plaintext)}`.
pub async fn store(
    redis: &mut ConnectionManager,
    plaintext_code: &str,
    data: &AuthCodeData,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let key = redis_key(&hash_code(plaintext_code));
    let value = serde_json::to_string(data).expect("AuthCodeData serialization cannot fail");
    redis.set_ex::<_, _, ()>(&key, &value, ttl_secs).await?;
    Ok(())
}

/// Look up and immediately delete an auth code (single-use).
/// Returns `None` if the code does not exist or has expired.
pub async fn consume(
    redis: &mut ConnectionManager,
    plaintext_code: &str,
) -> Result<Option<AuthCodeData>, redis::RedisError> {
    let key = redis_key(&hash_code(plaintext_code));

    // GET then DEL -- acceptable race window given 30s TTL
    let value: Option<String> = redis.get(&key).await?;
    if value.is_some() {
        redis.del::<_, ()>(&key).await?;
    }

    Ok(value.and_then(|v| serde_json::from_str(&v).ok()))
}
