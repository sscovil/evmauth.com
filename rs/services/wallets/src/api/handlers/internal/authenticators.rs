use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use turnkey_client::generated::immutable::activity::v1::{
    Attestation, AuthenticatorParamsV2, CreateAuthenticatorsIntentV2,
};
use turnkey_client::generated::services::coordinator::public::v1::GetUsersRequest;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::PasskeyAttestationParam;
use crate::repository::entity_wallet::{EntityWalletRepository, EntityWalletRepositoryImpl};

/// Request to add authenticators to an existing entity's Turnkey sub-org
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAuthenticatorsRequest {
    /// The entity ID (person) whose sub-org should receive the authenticators
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub entity_id: Uuid,

    /// Passkey authenticators to add
    pub authenticators: Vec<PasskeyAttestationParam>,
}

/// Add passkey authenticators to an existing entity's Turnkey sub-org
///
/// Looks up the entity wallet to find the sub-org, retrieves the root user,
/// then registers the new authenticators with Turnkey.
#[utoipa::path(
    post,
    path = "/internal/authenticators",
    request_body = CreateAuthenticatorsRequest,
    responses(
        (status = 201, description = "Authenticators added successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Entity wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/authenticators"
)]
pub async fn create_authenticators(
    State(state): State<AppState>,
    Json(body): Json<CreateAuthenticatorsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if body.authenticators.is_empty() {
        return Err(ApiError::BadRequest(
            "at least one authenticator is required".to_string(),
        ));
    }

    // Step 1: Look up entity wallet to get Turnkey sub-org ID
    let repo = EntityWalletRepositoryImpl::new(&state.db);
    let entity_wallet = repo
        .get_by_entity_id(body.entity_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let sub_org_id = entity_wallet.turnkey_sub_org_id.to_string();

    // Step 2: Get the root user ID from the sub-org
    let users_response = state
        .turnkey
        .get_users(GetUsersRequest {
            organization_id: sub_org_id.clone(),
        })
        .await?;

    let root_user = users_response
        .users
        .first()
        .ok_or_else(|| ApiError::Internal("no users found in sub-org".to_string()))?;

    // Step 3: Build authenticator params
    let authenticators: Vec<AuthenticatorParamsV2> = body
        .authenticators
        .iter()
        .map(|a| AuthenticatorParamsV2 {
            authenticator_name: a.authenticator_name.clone(),
            challenge: a.challenge.clone(),
            attestation: Some(Attestation {
                credential_id: String::new(),
                client_data_json: a.attestation.to_string(),
                attestation_object: String::new(),
                transports: vec![],
            }),
        })
        .collect();

    // Step 4: Register authenticators with Turnkey
    state
        .turnkey
        .create_authenticators(
            sub_org_id,
            state.turnkey.current_timestamp(),
            CreateAuthenticatorsIntentV2 {
                user_id: root_user.user_id.clone(),
                authenticators,
            },
        )
        .await?;

    Ok(StatusCode::CREATED)
}
