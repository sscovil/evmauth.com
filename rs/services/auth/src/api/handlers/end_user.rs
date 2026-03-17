use std::time::Duration;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::auth_code::{self, AuthCodeData};
use crate::dto::request::CreatePerson;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};

const WALLETS_SERVICE_TIMEOUT: Duration = Duration::from_secs(30);
const END_USER_TOKEN_DURATION_SECS: i64 = 3600;

// -- Request / response types --

#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct AuthorizeParams {
    /// The app's client_id from its registration
    pub client_id: String,
    /// The callback URL to redirect to after auth
    pub redirect_uri: String,
    /// Opaque state value forwarded back to the caller
    #[serde(default)]
    pub state: Option<String>,
    /// PKCE code challenge (S256)
    pub code_challenge: String,
    /// Must be "S256"
    pub code_challenge_method: String,
}

/// Returned by the authorize endpoint so the frontend can render the login form.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorizeResponse {
    pub app_name: String,
    pub client_id: String,
    pub redirect_uri: String,
}

/// Response from the registry internal API for app lookup.
#[derive(Debug, Deserialize)]
struct AppRegistrationInternal {
    id: Uuid,
    name: String,
    callback_urls: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthenticateRequest {
    /// The app's client_id
    pub client_id: String,
    /// Redirect URI (must match authorize step)
    pub redirect_uri: String,
    /// PKCE code challenge
    pub code_challenge: String,
    /// Opaque state value
    #[serde(default)]
    pub state: Option<String>,

    // -- End-user credentials --
    /// Email address
    pub email: String,
    /// Display name (used on first login / signup)
    pub display_name: Option<String>,

    /// Auth provider name (e.g. "turnkey")
    #[serde(default = "default_auth_provider")]
    pub auth_provider_name: String,
    /// Auth provider reference / user ID
    pub auth_provider_ref: Option<String>,
}

fn default_auth_provider() -> String {
    "turnkey".to_string()
}

/// Returned after successful authentication -- contains the redirect URL with auth code.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthenticateResponse {
    pub redirect_url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenRequest {
    /// Must be "authorization_code"
    pub grant_type: String,
    /// The plaintext authorization code
    pub code: String,
    /// PKCE code verifier
    pub code_verifier: String,
    /// Must match the redirect_uri from the authorize step
    pub redirect_uri: String,
    /// The app's client_id
    pub client_id: String,
}

/// Response from wallets internal entity-app-wallet endpoint.
#[derive(Debug, Deserialize)]
struct EntityAppWalletInternal {
    wallet_address: String,
}

/// Response from wallets internal entity-wallet endpoint.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EntityWalletInternal {
    wallet_address: String,
}

// -- Handlers --

/// Validate an end-user authorization request.
///
/// The frontend calls this to verify the client_id and redirect_uri before
/// showing the login form. Returns app info on success.
#[utoipa::path(
    get,
    path = "/authorize",
    params(
        ("client_id" = String, Query, description = "App client ID"),
        ("redirect_uri" = String, Query, description = "Callback URL"),
        ("code_challenge" = String, Query, description = "PKCE code challenge (S256)"),
        ("code_challenge_method" = String, Query, description = "Must be S256"),
        ("state" = Option<String>, Query, description = "Opaque state value")
    ),
    responses(
        (status = 200, description = "Authorization request valid", body = AuthorizeResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "end_user_auth"
)]
pub async fn authorize(
    State(state): State<AppState>,
    Query(params): Query<AuthorizeParams>,
) -> Result<Json<AuthorizeResponse>, ApiError> {
    if params.code_challenge_method != "S256" {
        return Err(ApiError::BadRequest(
            "code_challenge_method must be S256".to_string(),
        ));
    }

    let app = validate_client_and_redirect(&state, &params.client_id, &params.redirect_uri).await?;

    Ok(Json(AuthorizeResponse {
        app_name: app.name,
        client_id: params.client_id,
        redirect_uri: params.redirect_uri,
    }))
}

/// Authenticate an end user and issue an authorization code.
///
/// After the user submits credentials on the hosted login page, the frontend
/// calls this endpoint. On success it returns a redirect URL containing the
/// authorization code and state.
#[utoipa::path(
    post,
    path = "/authorize",
    request_body = AuthenticateRequest,
    responses(
        (status = 200, description = "Auth code issued", body = AuthenticateResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication failed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "end_user_auth"
)]
pub async fn authenticate(
    State(state): State<AppState>,
    Json(body): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, ApiError> {
    // Step 1: Validate client_id and redirect_uri
    let app = validate_client_and_redirect(&state, &body.client_id, &body.redirect_uri).await?;

    // Step 2: Find or create the end-user person
    let person_repo = PersonRepositoryImpl::new(&state.db);
    let person = find_or_create_end_user(
        &state,
        &person_repo,
        &body.email,
        body.display_name.as_deref(),
        &body.auth_provider_name,
        body.auth_provider_ref.as_deref(),
    )
    .await?;

    // Step 3: Ensure entity app wallet exists for (person, app)
    ensure_entity_app_wallet(&state, person.id, app.id).await?;

    // Step 4: Generate auth code and store in Redis
    let plaintext_code = auth_code::generate_code();
    let data = AuthCodeData {
        app_registration_id: app.id,
        person_id: person.id,
        code_challenge: body.code_challenge,
        redirect_uri: body.redirect_uri.clone(),
        client_id: body.client_id,
    };

    let ttl = state.config.auth_code_ttl_secs;
    auth_code::store(&mut state.redis.clone(), &plaintext_code, &data, ttl)
        .await
        .map_err(|e| ApiError::Internal(format!("failed to store auth code: {e}")))?;

    // Step 5: Build redirect URL
    let separator = if body.redirect_uri.contains('?') {
        '&'
    } else {
        '?'
    };
    let mut redirect_url = format!("{}{separator}code={plaintext_code}", body.redirect_uri);
    if let Some(s) = &body.state {
        redirect_url = format!("{redirect_url}&state={s}");
    }

    Ok(Json(AuthenticateResponse { redirect_url }))
}

/// Exchange an authorization code for an end-user JWT.
///
/// Validates the PKCE code_verifier against the stored code_challenge,
/// consumes the single-use auth code, and returns a signed JWT.
#[utoipa::path(
    post,
    path = "/tokens",
    request_body = TokenRequest,
    responses(
        (status = 200, description = "Token issued", body = crate::api::handlers::auth::TokenResponse),
        (status = 400, description = "Invalid request or PKCE verification failed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "end_user_auth"
)]
pub async fn token_exchange(
    State(state): State<AppState>,
    Json(body): Json<TokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if body.grant_type != "authorization_code" {
        return Err(ApiError::BadRequest(
            "grant_type must be authorization_code".to_string(),
        ));
    }

    let jwt_keys = state
        .jwt_keys
        .as_ref()
        .ok_or_else(|| ApiError::Internal("JWT not configured".to_string()))?;

    // Step 1: Consume the auth code from Redis (single-use)
    let code_data = auth_code::consume(&mut state.redis.clone(), &body.code)
        .await
        .map_err(|e| ApiError::Internal(format!("redis error: {e}")))?
        .ok_or_else(|| ApiError::BadRequest("invalid or expired authorization code".to_string()))?;

    // Step 2: Validate redirect_uri and client_id match
    if code_data.redirect_uri != body.redirect_uri {
        return Err(ApiError::BadRequest("redirect_uri mismatch".to_string()));
    }
    if code_data.client_id != body.client_id {
        return Err(ApiError::BadRequest("client_id mismatch".to_string()));
    }

    // Step 3: Verify PKCE -- SHA-256(code_verifier) must match stored code_challenge
    let computed_challenge = auth_code::pkce_s256(&body.code_verifier);
    if computed_challenge != code_data.code_challenge {
        return Err(ApiError::BadRequest(
            "PKCE code_verifier verification failed".to_string(),
        ));
    }

    // Step 4: Look up the end-user's app wallet address
    let wallet =
        get_entity_app_wallet(&state, code_data.person_id, code_data.app_registration_id).await?;

    // Step 5: Look up contract address for this app (if any)
    let contract_address = get_app_contract_address(&state, code_data.app_registration_id).await;

    // Step 6: Issue end-user JWT
    let chain_id = state.evm.config().chain_id.to_string();
    let token = crate::jwt::create_end_user_token(
        jwt_keys,
        code_data.person_id,
        &body.client_id,
        &wallet.wallet_address,
        &contract_address.unwrap_or_default(),
        &chain_id,
        END_USER_TOKEN_DURATION_SECS,
    )
    .map_err(|e| ApiError::Internal(format!("failed to create token: {e}")))?;

    Ok((
        StatusCode::OK,
        Json(super::auth::TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: END_USER_TOKEN_DURATION_SECS,
        }),
    ))
}

// -- Helper functions --

async fn validate_client_and_redirect(
    state: &AppState,
    client_id: &str,
    redirect_uri: &str,
) -> Result<AppRegistrationInternal, ApiError> {
    let wallets_url = &state.config.wallets_service_url;
    // The registry service is a separate service; derive URL from wallets URL pattern
    let registry_url = wallets_url.replace("int-wallets", "int-registry");

    let app: AppRegistrationInternal = tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
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
    .map_err(|_| ApiError::Internal("registry service request timed out".to_string()))??;

    // Validate redirect_uri BEFORE initiating auth (prevent open redirects)
    if !app.callback_urls.contains(&redirect_uri.to_string()) {
        return Err(ApiError::BadRequest(
            "redirect_uri is not registered for this app".to_string(),
        ));
    }

    Ok(app)
}

async fn find_or_create_end_user(
    state: &AppState,
    person_repo: &PersonRepositoryImpl<'_>,
    email: &str,
    display_name: Option<&str>,
    auth_provider_name: &str,
    auth_provider_ref: Option<&str>,
) -> Result<crate::domain::Person, ApiError> {
    // Try to find existing person by email
    let people = person_repo
        .list(
            crate::repository::PersonFilter::new()
                .email(email)
                .auth_provider(auth_provider_name),
            crate::repository::pagination::Page {
                first: Some(1),
                ..Default::default()
            },
        )
        .await?;

    if let Some(person) = people.into_iter().next() {
        return Ok(person);
    }

    // Create new person
    let display = display_name.unwrap_or_else(|| email.split('@').next().unwrap_or(email));
    let provider_ref = auth_provider_ref.unwrap_or("pending");

    let person = person_repo
        .create(CreatePerson {
            display_name: display.to_string(),
            description: None,
            auth_provider_name: auth_provider_name.to_string(),
            auth_provider_ref: provider_ref.to_string(),
            primary_email: email.to_string(),
        })
        .await?;

    // Create entity wallet for the new person
    let wallets_url = &state.config.wallets_service_url;
    let wallet_request = serde_json::json!({
        "entity_id": person.id,
        "sub_org_name": format!("person-{}", person.id),
        "root_user_name": email,
    });

    let resp = tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
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

    Ok(person)
}

async fn ensure_entity_app_wallet(
    state: &AppState,
    person_id: Uuid,
    app_registration_id: Uuid,
) -> Result<(), ApiError> {
    let wallets_url = &state.config.wallets_service_url;

    // Check if it already exists
    let check = tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
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
        return Ok(());
    }

    // Create it
    let create_request = serde_json::json!({
        "entity_id": person_id,
        "app_registration_id": app_registration_id,
    });

    let resp = tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
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

    Ok(())
}

async fn get_entity_app_wallet(
    state: &AppState,
    person_id: Uuid,
    app_registration_id: Uuid,
) -> Result<EntityAppWalletInternal, ApiError> {
    let wallets_url = &state.config.wallets_service_url;

    tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
        state
            .http_client
            .get(format!(
                "{wallets_url}/internal/entity-app-wallet/{person_id}/{app_registration_id}"
            ))
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("wallets service error: {e}")))?
            .error_for_status()
            .map_err(|e| ApiError::Internal(format!("entity app wallet lookup failed: {e}")))?
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("failed to parse wallet response: {e}")))
    })
    .await
    .map_err(|_| ApiError::Internal("wallets service timed out".to_string()))?
}

/// Best-effort lookup of the contract address for an app registration.
/// Returns None if the app has no deployed contract.
async fn get_app_contract_address(state: &AppState, app_registration_id: Uuid) -> Option<String> {
    let wallets_url = &state.config.wallets_service_url;
    let registry_url = wallets_url.replace("int-wallets", "int-registry");

    let resp = tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
        state
            .http_client
            .get(format!(
                "{registry_url}/internal/contracts/by-app/{app_registration_id}"
            ))
            .send()
            .await
    })
    .await
    .ok()?
    .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    #[derive(Deserialize)]
    struct ContractInternal {
        address: String,
    }

    resp.json::<ContractInternal>()
        .await
        .ok()
        .map(|c| c.address)
}
