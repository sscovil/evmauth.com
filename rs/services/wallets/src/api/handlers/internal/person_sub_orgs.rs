use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::CreatePersonTurnkeyRef;
use crate::dto::response::PersonTurnkeyRefResponse;
use crate::repository::person_turnkey_ref::{
    PersonTurnkeyRefRepository, PersonTurnkeyRefRepositoryImpl,
};

use turnkey::sub_org::{ApiKeyParams, AuthenticatorParams, CreateSubOrg, RootUser};

/// Create a Turnkey sub-org for a person
///
/// Creates a new Turnkey sub-organization and stores the reference
/// linking the person to their sub-org. Supports both API key and
/// passkey authenticator credentials for the root user.
#[utoipa::path(
    post,
    path = "/internal/person-sub-org",
    request_body = CreatePersonTurnkeyRef,
    responses(
        (status = 201, description = "Person sub-org created successfully", body = PersonTurnkeyRefResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/person_sub_orgs"
)]
pub async fn create_person_sub_org(
    State(state): State<AppState>,
    Json(create): Json<CreatePersonTurnkeyRef>,
) -> Result<impl IntoResponse, ApiError> {
    // Build API keys from optional fields
    let api_keys = match (create.api_key_name, create.api_public_key) {
        (Some(name), Some(key)) => vec![ApiKeyParams {
            api_key_name: name,
            public_key: key,
        }],
        _ => vec![],
    };

    // Convert passkey attestation params to Turnkey authenticator params
    let authenticators: Vec<AuthenticatorParams> = create
        .authenticators
        .into_iter()
        .map(|a| AuthenticatorParams {
            authenticator_name: a.authenticator_name,
            challenge: a.challenge,
            attestation: a.attestation,
        })
        .collect();

    if api_keys.is_empty() && authenticators.is_empty() {
        return Err(ApiError::BadRequest(
            "Either API key credentials or passkey authenticators must be provided".to_string(),
        ));
    }

    // Create the sub-org via Turnkey API
    let sub_org_response = state
        .turnkey
        .create_sub_org(CreateSubOrg {
            name: create.sub_org_name,
            root_users: vec![RootUser {
                user_name: create.root_user_name,
                api_keys,
                authenticators,
            }],
        })
        .await?;

    // Store the reference in the database
    let repo = PersonTurnkeyRefRepositoryImpl::new(&state.db);
    let ref_record = repo
        .create(create.person_id, &sub_org_response.sub_organization_id)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(PersonTurnkeyRefResponse::from(ref_record)),
    ))
}
