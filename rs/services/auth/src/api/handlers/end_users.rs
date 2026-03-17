use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::CreatePerson;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};

const SERVICE_TIMEOUT: Duration = Duration::from_secs(30);

/// Request to create or look up an end-user for an app.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEndUserRequest {
    /// End-user's email address
    #[schema(example = "user@example.com", format = "email")]
    pub email: String,
    /// Display name (optional, used on first signup)
    #[schema(example = "Alice")]
    pub display_name: Option<String>,
    /// Passkey attestation data (optional, for Turnkey-based signup)
    pub attestation: Option<super::auth::PasskeyAttestation>,
    /// App's client_id from its registration
    #[schema(example = "abc123def456")]
    pub client_id: String,
}

/// Response from end-user creation/lookup.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateEndUserResponse {
    /// Wallet address assigned to this end-user for the app
    #[schema(example = "0x1234...abcd")]
    pub wallet_address: String,
    /// Person ID of the end-user
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub person_id: Uuid,
}

/// Response from registry internal API for app lookup.
#[derive(Debug, Deserialize)]
struct AppRegistrationInternal {
    id: Uuid,
}

/// Response from wallets internal entity-app-wallet endpoint.
#[derive(Debug, Deserialize)]
struct EntityAppWalletInternal {
    wallet_address: String,
}

/// Create or look up an end-user and ensure they have a wallet for the given app.
///
/// Validates the client_id against the registry service, finds or creates the
/// person record, provisions wallets as needed, and returns the end-user's
/// app-specific wallet address.
#[utoipa::path(
    post,
    path = "/end-users",
    request_body = CreateEndUserRequest,
    responses(
        (status = 201, description = "End-user created", body = CreateEndUserResponse),
        (status = 200, description = "End-user already exists", body = CreateEndUserResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "end_users"
)]
pub async fn create_end_user(
    State(state): State<AppState>,
    Json(body): Json<CreateEndUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Validate client_id via registry service
    let app = validate_client_id(&state, &body.client_id).await?;

    // Step 2: Find or create the end-user person
    let person_repo = PersonRepositoryImpl::new(&state.db);
    let (person, created) = find_or_create_end_user(
        &state,
        &person_repo,
        &body.email,
        body.display_name.as_deref(),
        body.attestation.as_ref(),
    )
    .await?;

    // Step 3: Ensure entity app wallet exists for (person, app)
    let wallet = ensure_entity_app_wallet(&state, person.id, app.id).await?;

    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    Ok((
        status,
        Json(CreateEndUserResponse {
            wallet_address: wallet.wallet_address,
            person_id: person.id,
        }),
    ))
}

async fn validate_client_id(
    state: &AppState,
    client_id: &str,
) -> Result<AppRegistrationInternal, ApiError> {
    let wallets_url = &state.config.wallets_service_url;
    let registry_url = wallets_url.replace("int-wallets", "int-registry");

    tokio::time::timeout(SERVICE_TIMEOUT, async {
        state
            .http_client
            .get(format!(
                "{registry_url}/internal/apps/by-client-id/{client_id}"
            ))
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("failed to reach registry service: {e}")))?
            .error_for_status()
            .map_err(|e| {
                if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                    ApiError::BadRequest(format!("unknown client_id: {client_id}"))
                } else {
                    ApiError::Internal(format!("registry service error: {e}"))
                }
            })?
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("failed to parse registry response: {e}")))
    })
    .await
    .map_err(|_| ApiError::Internal("registry service request timed out".to_string()))?
}

async fn find_or_create_end_user(
    state: &AppState,
    person_repo: &PersonRepositoryImpl<'_>,
    email: &str,
    display_name: Option<&str>,
    attestation: Option<&super::auth::PasskeyAttestation>,
) -> Result<(crate::domain::Person, bool), ApiError> {
    // Try to find existing person by email
    let people = person_repo
        .list(
            crate::repository::PersonFilter::new()
                .email(email)
                .auth_provider("turnkey"),
            crate::repository::pagination::Page {
                first: Some(1),
                ..Default::default()
            },
        )
        .await?;

    if let Some(person) = people.into_iter().next() {
        return Ok((person, false));
    }

    // Create new person
    let display = display_name.unwrap_or_else(|| email.split('@').next().unwrap_or(email));

    let person = person_repo
        .create(CreatePerson {
            display_name: display.to_string(),
            description: None,
            auth_provider_name: "turnkey".to_string(),
            auth_provider_ref: "pending-turnkey".to_string(),
            primary_email: email.to_string(),
        })
        .await?;

    // Create entity wallet for the new person via wallets service
    let wallets_url = &state.config.wallets_service_url;
    let mut wallet_request = serde_json::json!({
        "entity_id": person.id,
        "sub_org_name": format!("person-{}", person.id),
        "root_user_name": email,
    });

    // Include passkey attestation if provided
    if let Some(att) = attestation {
        wallet_request["authenticators"] = serde_json::json!([{
            "authenticator_name": att.authenticator_name,
            "challenge": att.challenge,
            "attestation": att.attestation,
        }]);
    }

    let resp = tokio::time::timeout(SERVICE_TIMEOUT, async {
        state
            .http_client
            .post(format!("{wallets_url}/internal/entity-wallet"))
            .json(&wallet_request)
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
            "failed to create entity wallet: {status} {body}"
        )));
    }

    // Update auth_provider_ref to Turnkey sub-org ID
    #[derive(Deserialize)]
    struct WalletCreated {
        turnkey_sub_org_id: String,
    }
    if let Ok(wallet) =
        serde_json::from_str::<WalletCreated>(&resp.text().await.unwrap_or_else(|_| String::new()))
    {
        let _ = person_repo
            .update_auth_provider_ref(person.id, &wallet.turnkey_sub_org_id)
            .await;
    }

    Ok((person, true))
}

async fn ensure_entity_app_wallet(
    state: &AppState,
    person_id: Uuid,
    app_registration_id: Uuid,
) -> Result<EntityAppWalletInternal, ApiError> {
    let wallets_url = &state.config.wallets_service_url;

    // Check if it already exists
    let check = tokio::time::timeout(SERVICE_TIMEOUT, async {
        state
            .http_client
            .get(format!(
                "{wallets_url}/internal/entity-app-wallet/{person_id}/{app_registration_id}"
            ))
            .send()
            .await
    })
    .await
    .map_err(|_| ApiError::Internal("wallets service timed out".to_string()))?
    .map_err(|e| ApiError::Internal(format!("wallets service error: {e}")))?;

    if check.status().is_success() {
        return check
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("failed to parse wallet response: {e}")));
    }

    // Create it
    let create_request = serde_json::json!({
        "entity_id": person_id,
        "app_registration_id": app_registration_id,
    });

    let resp = tokio::time::timeout(SERVICE_TIMEOUT, async {
        state
            .http_client
            .post(format!("{wallets_url}/internal/entity-app-wallet"))
            .json(&create_request)
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
            "failed to create entity app wallet: {status} {body}"
        )));
    }

    resp.json()
        .await
        .map_err(|e| ApiError::Internal(format!("failed to parse wallet response: {e}")))
}
