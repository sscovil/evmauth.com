use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

use subtle::ConstantTimeEq;

use crate::api::error::ApiError;
use crate::domain::OrgVisibility;
use crate::dto::request::CreatePerson;
use crate::repository::filter::OrgFilter;
use crate::repository::org::{OrgRepository, OrgRepositoryImpl};
use crate::repository::pagination::Page;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};
use crate::{AppState, jwt};

/// Session token validity period in hours
const SESSION_DURATION_HOURS: i64 = 8;

/// Token ID for the API access capability token (ERC-6909)
const API_ACCESS_TOKEN_ID: u64 = 1;

/// Passkey attestation data from the frontend (via @turnkey/sdk-browser)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct PasskeyAttestation {
    /// Name for this authenticator
    #[schema(example = "my-passkey", format = "string")]
    pub authenticator_name: String,
    /// The challenge used during WebAuthn registration
    #[schema(example = "base64-encoded-challenge", format = "string")]
    pub challenge: String,
    /// The attestation object from WebAuthn
    pub attestation: serde_json::Value,
}

/// Deployer signup request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct SignupRequest {
    /// Display name for the new person
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: String,
    /// Primary email address
    #[schema(example = "alice.adams@example.com", format = "email")]
    pub primary_email: String,
    /// Passkey attestation data (for passkey-based signup via Turnkey)
    pub attestation: Option<PasskeyAttestation>,
    /// Authentication provider name (for OAuth-based signup, e.g. "google")
    #[schema(example = "turnkey", format = "string")]
    pub auth_provider_name: Option<String>,
    /// Authentication provider reference / user ID from provider
    #[schema(example = "usr_abc123xyz", format = "string")]
    pub auth_provider_ref: Option<String>,
}

/// Login request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    /// Primary email address
    #[schema(example = "alice.adams@example.com", format = "email")]
    pub primary_email: String,
    /// Authentication provider name
    #[schema(example = "turnkey", format = "string")]
    pub auth_provider_name: String,
    /// Authentication provider reference
    #[schema(example = "usr_abc123xyz", format = "string")]
    pub auth_provider_ref: String,
}

/// Token response
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    /// The JWT access token
    #[schema(example = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Token lifetime in seconds
    #[schema(example = 28800)]
    pub expires_in: i64,
}

/// Response from wallets service person sub-org creation
#[derive(Debug, serde::Deserialize)]
struct WalletsPersonSubOrgResponse {
    turnkey_sub_org_id: types::TurnkeySubOrgId,
}

/// Deployer signup
///
/// Creates a new person account, provisions a Turnkey sub-org and personal
/// workspace wallet, and issues a session JWT.
#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "Account created", body = TokenResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let jwt_keys = state
        .jwt_keys
        .as_ref()
        .ok_or_else(|| ApiError::Internal("JWT not configured".to_string()))?;

    // Validate: must have either passkey attestation or OAuth provider fields
    let (auth_provider_name, authenticators_json, api_key_name, api_public_key) =
        if let Some(attestation) = &body.attestation {
            // Passkey-based signup via Turnkey
            let authenticators = vec![serde_json::json!({
                "authenticator_name": attestation.authenticator_name,
                "challenge": attestation.challenge,
                "attestation": attestation.attestation,
            })];
            (
                "turnkey".to_string(),
                Some(authenticators),
                None::<String>,
                None::<String>,
            )
        } else if let (Some(provider_name), Some(_provider_ref)) =
            (&body.auth_provider_name, &body.auth_provider_ref)
        {
            // OAuth-based signup -- no Turnkey authenticators, will use API key flow
            (provider_name.clone(), None, None, None)
        } else {
            return Err(ApiError::BadRequest(
                "Either attestation or both auth_provider_name and auth_provider_ref are required"
                    .to_string(),
            ));
        };

    // Step 1: Call wallets service to create a Turnkey sub-org for this person
    // We use a placeholder person_id (will be replaced after person creation)
    // Actually, we need the person_id first for the sub-org, so we create the person first
    // but we need the sub-org ID for the person's auth_provider_ref...
    // Solution: create sub-org with a temporary person_id, then create person with the sub-org ID
    let wallets_url = &state.config.wallets_service_url;
    let temp_person_id = Uuid::new_v4();

    let mut sub_org_request = serde_json::json!({
        "person_id": temp_person_id,
        "sub_org_name": format!("person-{temp_person_id}"),
        "root_user_name": body.primary_email,
    });

    if let Some(authenticators) = authenticators_json {
        sub_org_request["authenticators"] = serde_json::Value::Array(authenticators);
    }

    if let Some(ref key_name) = api_key_name {
        sub_org_request["api_key_name"] = serde_json::Value::String(key_name.clone());
    }
    if let Some(ref pub_key) = api_public_key {
        sub_org_request["api_public_key"] = serde_json::Value::String(pub_key.clone());
    }

    let sub_org_response = state
        .http_client
        .post(format!("{wallets_url}/internal/person-sub-org"))
        .json(&sub_org_request)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to call wallets service: {e}")))?;

    if !sub_org_response.status().is_success() {
        let status = sub_org_response.status();
        let error_body = sub_org_response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        return Err(ApiError::Internal(format!(
            "Wallets service returned {status}: {error_body}"
        )));
    }

    let sub_org: WalletsPersonSubOrgResponse = sub_org_response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse wallets response: {e}")))?;

    // Step 2: Create person in auth.people
    // For passkey signup, auth_provider_ref is the Turnkey sub-org ID
    // For OAuth signup, use the provided auth_provider_ref
    let auth_provider_ref = if body.attestation.is_some() {
        sub_org.turnkey_sub_org_id.to_string()
    } else {
        body.auth_provider_ref
            .clone()
            .ok_or_else(|| ApiError::BadRequest("auth_provider_ref is required".to_string()))?
    };

    let person_repo = PersonRepositoryImpl::new(&state.db);
    let person = person_repo
        .create(CreatePerson {
            display_name: body.display_name,
            description: None,
            auth_provider_name,
            auth_provider_ref,
            primary_email: body.primary_email,
        })
        .await?;

    // Step 3: Find the auto-created personal workspace org
    let org_repo = OrgRepositoryImpl::new(&state.db);
    let personal_orgs = org_repo
        .list(
            OrgFilter::new()
                .owner_id(person.id)
                .visibility(OrgVisibility::Personal),
            Page {
                first: Some(1),
                ..Default::default()
            },
        )
        .await?;

    let personal_org = personal_orgs.first().ok_or_else(|| {
        ApiError::Internal("Personal workspace org was not auto-created".to_string())
    })?;

    // Step 4: Call wallets service to create an org wallet for the personal workspace
    let org_wallet_request = serde_json::json!({
        "org_id": personal_org.id,
        "sub_org_name": format!("org-{}", personal_org.id),
    });

    let org_wallet_response = state
        .http_client
        .post(format!("{wallets_url}/internal/org-wallet"))
        .json(&org_wallet_request)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create org wallet: {e}")))?;

    if !org_wallet_response.status().is_success() {
        let status = org_wallet_response.status();
        let error_body = org_wallet_response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        tracing::error!("Failed to create org wallet: {status} {error_body}");
        // Log but don't fail signup -- the wallet can be created later
    } else {
        // Step 4b: Mint capability token for the new org wallet (best-effort)
        if let Ok(wallet_info) = org_wallet_response.json::<serde_json::Value>().await
            && let Some(wallet_address_str) = wallet_info.get("address").and_then(|v| v.as_str())
        {
            match wallet_address_str.parse::<evm::Address>() {
                Ok(wallet_address) => {
                    // TOKEN_ID 1 = API access capability token
                    let token_id = evm::U256::from(API_ACCESS_TOKEN_ID);
                    let amount = evm::U256::from(1);
                    let calldata = evm::EvmClient::encode_mint(wallet_address, token_id, amount);

                    let mint_request = serde_json::json!({
                        "to": format!("{}", state.evm.platform_contract_address()),
                        "data": format!("{calldata}"),
                        "chain_id": state.evm.config().chain_id,
                    });

                    match state
                        .http_client
                        .post(format!("{wallets_url}/internal/sign"))
                        .json(&mint_request)
                        .send()
                        .await
                    {
                        Ok(resp) if resp.status().is_success() => {
                            tracing::info!(
                                org_id = %personal_org.id,
                                wallet = wallet_address_str,
                                "Capability token mint submitted"
                            );
                        }
                        Ok(resp) => {
                            let status = resp.status();
                            let body = resp
                                .text()
                                .await
                                .unwrap_or_else(|_| "unknown error".to_string());
                            tracing::warn!(
                                org_id = %personal_org.id,
                                "Capability token mint failed: {status} {body}"
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                org_id = %personal_org.id,
                                "Capability token mint request failed: {e}"
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        org_id = %personal_org.id,
                        "Invalid wallet address from wallets service: {e}"
                    );
                }
            }
        }
    }

    // Step 5: Issue session JWT
    let token = jwt::create_session_token(jwt_keys, person.id, SESSION_DURATION_HOURS)
        .map_err(|e| ApiError::Internal(format!("Failed to create token: {e}")))?;

    let cookie = format!(
        "session={token}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        SESSION_DURATION_HOURS * 3600
    );

    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie)],
        Json(TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: SESSION_DURATION_HOURS * 3600,
        }),
    ))
}

/// Deployer login
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let jwt_keys = state
        .jwt_keys
        .as_ref()
        .ok_or_else(|| ApiError::Internal("JWT not configured".to_string()))?;

    let repo = PersonRepositoryImpl::new(&state.db);

    // Look up person by email and auth provider
    let people = repo
        .list(
            crate::repository::PersonFilter::new()
                .email(&body.primary_email)
                .auth_provider(&body.auth_provider_name),
            Page {
                first: Some(1),
                ..Default::default()
            },
        )
        .await?;

    let person = people.first().ok_or(ApiError::NotFound)?;

    // Verify auth provider ref matches (constant-time to prevent timing attacks)
    if person
        .auth_provider_ref
        .as_bytes()
        .ct_eq(body.auth_provider_ref.as_bytes())
        .into()
    {
        // match -- fall through
    } else {
        return Err(ApiError::BadRequest("Invalid credentials".to_string()));
    }

    let token = jwt::create_session_token(jwt_keys, person.id, SESSION_DURATION_HOURS)
        .map_err(|e| ApiError::Internal(format!("Failed to create token: {e}")))?;

    let cookie = format!(
        "session={token}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        SESSION_DURATION_HOURS * 3600
    );

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: SESSION_DURATION_HOURS * 3600,
        }),
    ))
}

/// Logout - clears the session cookie
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logged out")
    ),
    tag = "auth"
)]
pub async fn logout() -> impl IntoResponse {
    let cookie = "session=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0";
    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(json!({"status": "logged out"})),
    )
}
