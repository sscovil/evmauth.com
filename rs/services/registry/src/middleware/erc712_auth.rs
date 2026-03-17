use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::AppState;

/// Maximum age for a nonce timestamp (seconds).
const MAX_NONCE_AGE_SECS: i64 = 30;

/// Token ID for the API access capability token (ERC-6909).
const API_ACCESS_TOKEN_ID: u64 = 1;

/// Context extracted by the ERC-712 auth middleware, available to downstream handlers.
#[derive(Debug, Clone)]
pub struct Erc712AuthContext {
    pub signer: evm::Address,
    pub client_id: String,
}

/// Middleware that verifies ERC-712 signed requests for the `/accounts` endpoint.
///
/// Extracts headers:
/// - `X-Client-Id`: The app's client_id
/// - `X-Signer`: The wallet address claiming to be the signer
/// - `X-Signature`: Hex-encoded ERC-712 signature
/// - `X-Nonce`: Hex-encoded 32-byte nonce (must be recent timestamp-based)
/// - `X-Contract`: The contract address being queried
///
/// Verifies:
/// 1. Signature recovers to the claimed signer
/// 2. Nonce is recent (within MAX_NONCE_AGE_SECS)
/// 3. Signer holds the API access capability token on the platform contract
pub async fn require_erc712_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let client_id = match get_header(&request, "x-client-id") {
        Some(v) => v,
        None => return error_response(StatusCode::BAD_REQUEST, "missing X-Client-Id header"),
    };

    let signer_str = match get_header(&request, "x-signer") {
        Some(v) => v,
        None => return error_response(StatusCode::BAD_REQUEST, "missing X-Signer header"),
    };

    let signature_hex = match get_header(&request, "x-signature") {
        Some(v) => v,
        None => return error_response(StatusCode::BAD_REQUEST, "missing X-Signature header"),
    };

    let nonce_hex = match get_header(&request, "x-nonce") {
        Some(v) => v,
        None => return error_response(StatusCode::BAD_REQUEST, "missing X-Nonce header"),
    };

    let contract_str = match get_header(&request, "x-contract") {
        Some(v) => v,
        None => return error_response(StatusCode::BAD_REQUEST, "missing X-Contract header"),
    };

    // Parse addresses
    let signer: evm::Address = match signer_str.parse() {
        Ok(a) => a,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "invalid X-Signer address"),
    };

    let contract: evm::Address = match contract_str.parse() {
        Ok(a) => a,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "invalid X-Contract address"),
    };

    // Parse nonce
    let nonce_bytes = match hex::decode(nonce_hex.strip_prefix("0x").unwrap_or(&nonce_hex)) {
        Ok(b) if b.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&b);
            evm::FixedBytes::from(arr)
        }
        _ => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid X-Nonce (expected 32 bytes hex)",
            );
        }
    };

    // Validate nonce freshness: first 8 bytes are a Unix timestamp
    let nonce_timestamp = i64::from_be_bytes(nonce_bytes[..8].try_into().unwrap_or_default());
    let now = chrono::Utc::now().timestamp();
    if (now - nonce_timestamp).abs() > MAX_NONCE_AGE_SECS {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "nonce is too old or from the future",
        );
    }

    // Parse signature
    let sig_bytes = match hex::decode(signature_hex.strip_prefix("0x").unwrap_or(&signature_hex)) {
        Ok(b) => b,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "invalid X-Signature hex"),
    };

    // Verify ERC-712 signature
    let chain_id = state.evm.config().chain_id;
    let recovered = match evm::verify_accounts_query(
        signer,
        contract,
        &client_id,
        nonce_bytes,
        chain_id,
        &sig_bytes,
    ) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::warn!("ERC-712 verification failed: {e}");
            return error_response(StatusCode::UNAUTHORIZED, "signature verification failed");
        }
    };

    if recovered != signer {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "recovered address does not match X-Signer",
        );
    }

    // Verify signer holds API access token on the platform contract
    let token_id = evm::U256::from(API_ACCESS_TOKEN_ID);
    match state.evm.balance_of(signer, token_id).await {
        Ok(balance) if !balance.is_zero() => {}
        Ok(_) => {
            return error_response(
                StatusCode::FORBIDDEN,
                "signer does not hold API access token",
            );
        }
        Err(e) => {
            tracing::warn!("balance_of check failed for {signer}: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "on-chain authorization check failed",
            );
        }
    }

    request
        .extensions_mut()
        .insert(Erc712AuthContext { signer, client_id });

    next.run(request).await
}

fn get_header(request: &Request, name: &str) -> Option<String> {
    request
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error": message}))).into_response()
}
