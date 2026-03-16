use std::time::Duration;

use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::PersonResponse;
use crate::middleware::AuthenticatedPerson;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};

const WALLETS_SERVICE_TIMEOUT: Duration = Duration::from_secs(30);

/// Get current person profile
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "Current person profile", body = PersonResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Person not found")
    ),
    tag = "me",
    security(("session" = []))
)]
pub async fn get_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedPerson>,
) -> Result<Json<PersonResponse>, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo.get(auth.person_id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(person.into()))
}

/// Update display name request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateMeRequest {
    /// New display name
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: Option<String>,
    /// New description
    #[schema(
        example = "Software engineer and open source contributor",
        format = "string"
    )]
    pub description: Option<String>,
}

/// Update current person profile
#[utoipa::path(
    patch,
    path = "/me",
    request_body = UpdateMeRequest,
    responses(
        (status = 200, description = "Profile updated", body = PersonResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Person not found")
    ),
    tag = "me",
    security(("session" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedPerson>,
    Json(body): Json<UpdateMeRequest>,
) -> Result<Json<PersonResponse>, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo
        .update(
            auth.person_id,
            crate::dto::request::UpdatePerson {
                display_name: body.display_name,
                description: body.description,
                primary_email: None,
            },
        )
        .await?;
    Ok(Json(person.into()))
}

/// Request to register a backup passkey authenticator
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAuthenticatorRequest {
    /// Name for this authenticator
    #[schema(example = "my-backup-passkey", format = "string")]
    pub authenticator_name: String,
    /// The challenge used during the WebAuthn registration ceremony
    #[schema(example = "base64-encoded-challenge", format = "string")]
    pub challenge: String,
    /// The attestation object from WebAuthn
    pub attestation: serde_json::Value,
}

/// Register a backup passkey authenticator for the current user
///
/// Calls the wallets service to add the passkey to the user's Turnkey sub-org.
#[utoipa::path(
    post,
    path = "/me/authenticators",
    request_body = CreateAuthenticatorRequest,
    responses(
        (status = 201, description = "Authenticator registered successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Internal server error")
    ),
    tag = "me",
    security(("session" = []))
)]
pub async fn create_authenticator(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedPerson>,
    Json(body): Json<CreateAuthenticatorRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let wallets_url = &state.config.wallets_service_url;

    let request_body = serde_json::json!({
        "entity_id": auth.person_id,
        "authenticators": [{
            "authenticator_name": body.authenticator_name,
            "challenge": body.challenge,
            "attestation": body.attestation,
        }],
    });

    let resp = tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
        state
            .http_client
            .post(format!("{wallets_url}/internal/authenticators"))
            .json(&request_body)
            .send()
            .await
    })
    .await
    .map_err(|_| ApiError::Internal("wallets service timed out".to_string()))?
    .map_err(|e| ApiError::Internal(format!("wallets service error: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|_| "unknown".to_string());
        return Err(ApiError::Internal(format!(
            "failed to create authenticator: {status} {body}"
        )));
    }

    Ok(StatusCode::CREATED)
}
