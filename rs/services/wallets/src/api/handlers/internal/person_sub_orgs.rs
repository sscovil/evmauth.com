use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use turnkey_client::generated::immutable::activity::v1::{
    ApiKeyParamsV2, Attestation, AuthenticatorParamsV2, CreateSubOrganizationIntentV7,
    RootUserParamsV4,
};
use turnkey_client::generated::immutable::common::v1::ApiKeyCurve;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::CreatePersonTurnkeyRef;
use crate::dto::response::PersonTurnkeyRefResponse;
use crate::repository::person_turnkey_ref::{
    PersonTurnkeyRefRepository, PersonTurnkeyRefRepositoryImpl,
};

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
    let api_keys: Vec<ApiKeyParamsV2> = match (create.api_key_name, create.api_public_key) {
        (Some(name), Some(key)) => vec![ApiKeyParamsV2 {
            api_key_name: name,
            public_key: key,
            curve_type: ApiKeyCurve::P256,
            expiration_seconds: None,
        }],
        _ => vec![],
    };

    let authenticators: Vec<AuthenticatorParamsV2> = create
        .authenticators
        .into_iter()
        .map(|a| AuthenticatorParamsV2 {
            authenticator_name: a.authenticator_name,
            challenge: a.challenge,
            attestation: Some(Attestation {
                credential_id: String::new(),
                client_data_json: a.attestation.to_string(),
                attestation_object: String::new(),
                transports: vec![],
            }),
        })
        .collect();

    if api_keys.is_empty() && authenticators.is_empty() {
        return Err(ApiError::BadRequest(
            "either API key credentials or passkey authenticators must be provided".to_string(),
        ));
    }

    let result = state
        .turnkey
        .create_sub_organization(
            state.turnkey_parent_org_id.clone(),
            state.turnkey.current_timestamp(),
            CreateSubOrganizationIntentV7 {
                sub_organization_name: create.sub_org_name,
                root_users: vec![RootUserParamsV4 {
                    user_name: create.root_user_name,
                    user_email: None,
                    user_phone_number: None,
                    api_keys,
                    authenticators,
                    oauth_providers: vec![],
                }],
                root_quorum_threshold: 1,
                wallet: None,
                disable_email_recovery: None,
                disable_email_auth: None,
                disable_sms_auth: None,
                disable_otp_email_auth: None,
                verification_token: None,
                client_signature: None,
            },
        )
        .await?;

    let repo = PersonTurnkeyRefRepositoryImpl::new(&state.db);
    let ref_record = repo
        .create(create.person_id, &result.result.sub_organization_id)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(PersonTurnkeyRefResponse::from(ref_record)),
    ))
}
