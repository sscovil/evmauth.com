use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::api::error::ApiError;

#[derive(Debug, Serialize, ToSchema)]
pub struct JwksResponse {
    pub keys: Vec<serde_json::Value>,
}

/// Serve the JSON Web Key Set for JWT verification.
///
/// Deployer apps use this endpoint to obtain the RS256 public key
/// for offline verification of end-user JWTs.
#[utoipa::path(
    get,
    path = "/.well-known/jwks.json",
    responses(
        (status = 200, description = "JWKS", body = JwksResponse),
        (status = 500, description = "JWT not configured")
    ),
    tag = "end_user_auth"
)]
pub async fn jwks(State(state): State<AppState>) -> Result<Json<JwksResponse>, ApiError> {
    let public_pem = state
        .config
        .jwt_public_key_pem
        .as_ref()
        .ok_or_else(|| ApiError::Internal("JWT public key not configured".to_string()))?;

    let jwk = pem_to_jwk(public_pem)
        .map_err(|e| ApiError::Internal(format!("failed to convert PEM to JWK: {e}")))?;

    Ok(Json(JwksResponse { keys: vec![jwk] }))
}

/// Convert an RSA PEM public key to a JWK JSON value.
fn pem_to_jwk(pem_str: &str) -> Result<serde_json::Value, String> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use rsa::pkcs8::DecodePublicKey;
    use rsa::traits::PublicKeyParts;

    let public_key = rsa::RsaPublicKey::from_public_key_pem(pem_str)
        .map_err(|e| format!("invalid RSA public key PEM: {e}"))?;

    let n = URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e = URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

    Ok(serde_json::json!({
        "kty": "RSA",
        "use": "sig",
        "alg": "RS256",
        "n": n,
        "e": e,
    }))
}
