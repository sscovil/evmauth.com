use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::CreatePersonTurnkeyRef;
use crate::dto::response::PersonTurnkeyRefResponse;
use crate::repository::person_turnkey_ref::{
    PersonTurnkeyRefRepository, PersonTurnkeyRefRepositoryImpl,
};

use turnkey::sub_org::{ApiKeyParams, CreateSubOrg, RootUser};

/// Create a Turnkey sub-org for a person
///
/// Creates a new Turnkey sub-organization and stores the reference
/// linking the person to their sub-org.
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
    // Create the sub-org via Turnkey API
    let sub_org_response = state
        .turnkey
        .create_sub_org(CreateSubOrg {
            name: create.sub_org_name,
            root_users: vec![RootUser {
                user_name: create.root_user_name,
                api_keys: vec![ApiKeyParams {
                    api_key_name: create.api_key_name,
                    public_key: create.api_public_key,
                }],
                authenticators: vec![],
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
