use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::repository::org_wallet::{OrgWalletRepository, OrgWalletRepositoryImpl};

use turnkey::signing::{HashFunction, PayloadEncoding, SignRawPayloadParams};

/// Request to sign a payload via delegated account
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignRequest {
    /// The organization ID whose delegated account should sign
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Uuid,

    /// The payload to sign (hex-encoded)
    #[schema(example = "0xdeadbeef", format = "string")]
    pub payload: String,

    /// The encoding of the payload
    #[schema(example = "hexadecimal", format = "string")]
    pub encoding: String,

    /// The hash function to use
    #[schema(example = "keccak256", format = "string")]
    pub hash_function: String,
}

/// Response containing the signature components
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignResponse {
    /// The r component of the signature
    pub r: String,
    /// The s component of the signature
    pub s: String,
    /// The v component of the signature
    pub v: String,
}

/// Sign a payload via a delegated account
///
/// Looks up the org's delegated account and signs the given payload
/// using the Turnkey API.
#[utoipa::path(
    post,
    path = "/internal/sign",
    request_body = SignRequest,
    responses(
        (status = 200, description = "Payload signed successfully", body = SignResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Org wallet or delegated account not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/signing"
)]
pub async fn sign_payload(
    State(state): State<AppState>,
    Json(request): Json<SignRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Look up the org wallet to get the sub-org and delegated user
    let repo = OrgWalletRepositoryImpl::new(&state.db);
    let org_wallet = repo
        .get_by_org_id(request.org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let delegated_user_id = org_wallet.turnkey_delegated_user_id.ok_or_else(|| {
        ApiError::BadRequest(format!(
            "Organization {} does not have a delegated signing account",
            request.org_id,
        ))
    })?;

    // Step 2: Parse encoding and hash function
    let encoding = match request.encoding.as_str() {
        "hexadecimal" => PayloadEncoding::PayloadEncodingHexadecimal,
        "utf8" => PayloadEncoding::PayloadEncodingUtf8,
        other => {
            return Err(ApiError::BadRequest(format!(
                "Unsupported encoding: {other}. Use 'hexadecimal' or 'utf8'",
            )));
        }
    };

    let hash_function = match request.hash_function.as_str() {
        "no_op" => HashFunction::HashFunctionNoOp,
        "sha256" => HashFunction::HashFunctionSha256,
        "keccak256" => HashFunction::HashFunctionKeccak256,
        other => {
            return Err(ApiError::BadRequest(format!(
                "Unsupported hash function: {other}. Use 'no_op', 'sha256', or 'keccak256'",
            )));
        }
    };

    // Step 3: Sign via Turnkey API
    let signature = state
        .turnkey
        .sign_raw_payload(SignRawPayloadParams {
            sub_org_id: org_wallet.turnkey_sub_org_id,
            user_id: delegated_user_id,
            payload: request.payload,
            encoding,
            hash_function,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(SignResponse {
            r: signature.r,
            s: signature.s,
            v: signature.v,
        }),
    ))
}
