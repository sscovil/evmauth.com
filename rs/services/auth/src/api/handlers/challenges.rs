use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use redis::AsyncCommands;
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::api::error::ApiError;

/// TTL for challenge nonces in Redis (seconds).
const CHALLENGE_TTL_SECS: u64 = 60;

/// Response containing a challenge nonce for signature-based authentication.
#[derive(Debug, Serialize, ToSchema)]
pub struct ChallengeResponse {
    /// Hex-encoded 32-byte random nonce
    #[schema(example = "a1b2c3d4e5f6...")]
    pub challenge: String,
}

fn redis_key(challenge: &str) -> String {
    format!("challenge:{challenge}")
}

/// Create a challenge nonce for deployer login.
///
/// Returns a 32-byte hex-encoded nonce stored in Redis with a 60-second TTL.
/// The client must sign this nonce with their wallet's private key and submit
/// it to `POST /sessions` to authenticate.
#[utoipa::path(
    post,
    path = "/challenges",
    responses(
        (status = 201, description = "Challenge created", body = ChallengeResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn create_challenge(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let nonce: [u8; 32] = rand::random();
    let challenge = hex::encode(nonce);

    let key = redis_key(&challenge);
    state
        .redis
        .clone()
        .set_ex::<_, _, ()>(&key, "1", CHALLENGE_TTL_SECS)
        .await
        .map_err(|e| ApiError::Internal(format!("failed to store challenge: {e}")))?;

    Ok((StatusCode::CREATED, Json(ChallengeResponse { challenge })))
}

/// Consume a challenge nonce from Redis (single-use).
/// Returns `true` if the nonce existed and was consumed, `false` otherwise.
pub async fn consume_challenge(
    redis: &mut redis::aio::ConnectionManager,
    challenge: &str,
) -> Result<bool, ApiError> {
    let key = redis_key(challenge);
    let value: Option<String> = redis
        .get(&key)
        .await
        .map_err(|e| ApiError::Internal(format!("redis error: {e}")))?;

    if value.is_some() {
        redis
            .del::<_, ()>(&key)
            .await
            .map_err(|e| ApiError::Internal(format!("redis error: {e}")))?;
        Ok(true)
    } else {
        Ok(false)
    }
}
