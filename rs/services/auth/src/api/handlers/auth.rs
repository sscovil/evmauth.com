use std::time::Duration;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;

use crate::api::error::ApiError;
use crate::api::handlers::challenges;
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

/// Timeout for internal service calls
const SERVICE_TIMEOUT: Duration = Duration::from_secs(30);

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

/// Deployer signup request (passkey-only)
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct SignupRequest {
    /// Display name for the new person
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: String,
    /// Primary email address
    #[schema(example = "alice.adams@example.com", format = "email")]
    pub primary_email: String,
    /// Passkey attestation data (required -- passkey-only signup)
    pub attestation: PasskeyAttestation,
}

/// Login request -- wallet signature verification
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    /// Wallet address of the deployer
    #[schema(example = "0x1234...abcd", format = "string")]
    pub wallet_address: String,
    /// Hex-encoded signature of the challenge nonce
    #[schema(example = "0xabc123...", format = "string")]
    pub signature: String,
    /// The challenge nonce from `POST /challenges`
    #[schema(example = "a1b2c3d4e5f6...", format = "string")]
    pub challenge: String,
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

/// Response from wallets service entity wallet creation
#[derive(Debug, serde::Deserialize)]
struct EntityWalletResponse {
    turnkey_sub_org_id: types::TurnkeySubOrgId,
    wallet_address: types::ChecksumAddress,
}

/// Response from wallets service wallet-by-address lookup
#[derive(Debug, serde::Deserialize)]
struct WalletByAddressResponse {
    entity_id: uuid::Uuid,
}

/// Deployer signup (passkey-only)
///
/// Creates a new person account, provisions a Turnkey sub-org and personal
/// workspace wallet, and issues a session JWT.
#[utoipa::path(
    post,
    path = "/people",
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

    let wallets_url = &state.config.wallets_service_url;

    // Step 1: Create person with temporary auth_provider_ref (Turnkey sub-org ID set after wallet creation)
    let authenticators = vec![serde_json::json!({
        "authenticator_name": body.attestation.authenticator_name,
        "challenge": body.attestation.challenge,
        "attestation": body.attestation.attestation,
    })];

    let person_repo = PersonRepositoryImpl::new(&state.db);
    let person = person_repo
        .create(CreatePerson {
            display_name: body.display_name,
            description: None,
            auth_provider_name: "turnkey".to_string(),
            auth_provider_ref: "pending-turnkey".to_string(),
            primary_email: body.primary_email,
        })
        .await?;

    // Step 2: Create entity wallet for the person (Turnkey sub-org + HD wallet)
    let person_wallet_request = serde_json::json!({
        "entity_id": person.id,
        "sub_org_name": format!("person-{}", person.id),
        "root_user_name": person.primary_email,
        "authenticators": authenticators,
    });

    let person_wallet_response = state
        .http_client
        .post(format!("{wallets_url}/internal/entity-wallet"))
        .json(&person_wallet_request)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to call wallets service: {e}")))?;

    if !person_wallet_response.status().is_success() {
        let status = person_wallet_response.status();
        let error_body = person_wallet_response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        return Err(ApiError::Internal(format!(
            "Wallets service returned {status}: {error_body}"
        )));
    }

    let person_wallet: EntityWalletResponse = person_wallet_response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse wallets response: {e}")))?;

    // Step 2b: Update auth_provider_ref to the Turnkey sub-org ID
    person_repo
        .update_auth_provider_ref(person.id, &person_wallet.turnkey_sub_org_id.to_string())
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

    // Step 4: Create entity wallet for the personal workspace org (with delegated account)
    let org_wallet_request = serde_json::json!({
        "entity_id": personal_org.id,
        "sub_org_name": format!("org-{}", personal_org.id),
        "delegated_user_name": format!("delegated-{}", personal_org.id),
        "delegated_api_public_key": state.config.delegated_api_public_key,
    });

    let org_wallet_response = state
        .http_client
        .post(format!("{wallets_url}/internal/entity-wallet"))
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
        if let Ok(org_wallet) = org_wallet_response.json::<EntityWalletResponse>().await {
            mint_capability_token(&state, personal_org.id, &org_wallet).await;
        }
    }

    // Step 5: Issue session JWT with wallet claim
    let token = jwt::create_session_token(
        jwt_keys,
        person.id,
        person_wallet.wallet_address.as_str(),
        SESSION_DURATION_HOURS,
    )
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

/// Deployer login via wallet signature
///
/// Verifies a signed challenge nonce against the wallet address, confirms
/// the wallet holds the platform API access capability token, and issues
/// a session JWT.
#[utoipa::path(
    post,
    path = "/sessions",
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

    // Step 1: Consume challenge nonce from Redis (single-use)
    let valid = challenges::consume_challenge(&mut state.redis.clone(), &body.challenge).await?;
    if !valid {
        return Err(ApiError::Unauthorized(
            "invalid or expired challenge".to_string(),
        ));
    }

    // Step 2: Recover signer from the signed challenge
    let sig_bytes = hex::decode(body.signature.strip_prefix("0x").unwrap_or(&body.signature))
        .map_err(|e| ApiError::BadRequest(format!("invalid signature hex: {e}")))?;

    let recovered = evm::recover_signer(body.challenge.as_bytes(), &sig_bytes)
        .map_err(|e| ApiError::Unauthorized(format!("signature verification failed: {e}")))?;

    // Step 3: Verify recovered address matches claimed wallet_address
    let claimed: evm::Address = body
        .wallet_address
        .parse()
        .map_err(|e| ApiError::BadRequest(format!("invalid wallet_address: {e}")))?;

    if recovered != claimed {
        return Err(ApiError::Unauthorized(
            "signature does not match wallet_address".to_string(),
        ));
    }

    // Step 4: Look up person by wallet address via wallets service
    let wallets_url = &state.config.wallets_service_url;
    let wallet_lookup = tokio::time::timeout(SERVICE_TIMEOUT, async {
        state
            .http_client
            .get(format!(
                "{wallets_url}/internal/entity-wallet-by-address/{claimed}"
            ))
            .send()
            .await
    })
    .await
    .map_err(|_| ApiError::Internal("wallets service timed out".to_string()))?
    .map_err(|e| ApiError::Internal(format!("wallets service error: {e}")))?;

    if !wallet_lookup.status().is_success() {
        return Err(ApiError::Unauthorized(
            "no account found for this wallet address".to_string(),
        ));
    }

    let wallet_data: WalletByAddressResponse = wallet_lookup
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("failed to parse wallet lookup: {e}")))?;

    // Step 5: Verify the wallet holds the API access capability token
    let token_id = evm::U256::from(API_ACCESS_TOKEN_ID);
    let balance = state.evm.balance_of(claimed, token_id).await.map_err(|e| {
        tracing::warn!("balance_of check failed for {claimed}: {e}");
        ApiError::Internal("on-chain authorization check failed".to_string())
    })?;

    if balance.is_zero() {
        return Err(ApiError::Unauthorized(
            "wallet does not hold API access token".to_string(),
        ));
    }

    // Step 6: Look up person record
    let person_repo = PersonRepositoryImpl::new(&state.db);
    let person = person_repo
        .get(wallet_data.entity_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Step 7: Issue session JWT with wallet claim
    let token = jwt::create_session_token(
        jwt_keys,
        person.id,
        &body.wallet_address,
        SESSION_DURATION_HOURS,
    )
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
    delete,
    path = "/sessions",
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

/// Best-effort mint of the API access capability token for a newly created org wallet.
async fn mint_capability_token(
    state: &AppState,
    org_id: uuid::Uuid,
    org_wallet: &EntityWalletResponse,
) {
    let wallets_url = &state.config.wallets_service_url;

    match org_wallet.wallet_address.as_str().parse::<evm::Address>() {
        Ok(wallet_address) => {
            let token_id = evm::U256::from(API_ACCESS_TOKEN_ID);
            let amount = evm::U256::from(1);
            let calldata = evm::EvmClient::encode_mint(wallet_address, token_id, amount);

            let mint_request = serde_json::json!({
                "entity_id": org_id,
                "to": format!("{}", state.evm.platform_contract_address()),
                "data": format!("{calldata}"),
                "chain_id": state.evm.config().chain_id,
            });

            match state
                .http_client
                .post(format!("{wallets_url}/internal/signatures"))
                .json(&mint_request)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!(
                        org_id = %org_id,
                        wallet = %org_wallet.wallet_address,
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
                        org_id = %org_id,
                        "Capability token mint failed: {status} {body}"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        org_id = %org_id,
                        "Capability token mint request failed: {e}"
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                org_id = %org_id,
                "Invalid wallet address from wallets service: {e}"
            );
        }
    }
}
