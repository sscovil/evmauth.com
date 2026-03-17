use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use turnkey_client::generated::immutable::activity::v1::{
    ApiKeyParamsV2, Attestation, AuthenticatorParamsV2, CreatePolicyIntentV3,
    CreateSubOrganizationIntentV7, CreateUsersIntentV3, CreateWalletIntent, RootUserParamsV4,
    UserParamsV3, WalletAccountParams,
};
use turnkey_client::generated::immutable::common::v1::{
    AddressFormat, ApiKeyCurve, Curve, Effect, PathFormat,
};

use types::{ChecksumAddress, TurnkeySubOrgId};

use crate::AppState;

/// BIP-32 derivation path for Ethereum accounts (EIP-44)
const ETH_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";
use crate::api::error::ApiError;
use crate::dto::request::CreateEntityWallet;
use crate::dto::response::EntityWalletResponse;
use crate::repository::entity_wallet::{EntityWalletRepository, EntityWalletRepositoryImpl};

/// Create an entity wallet with Turnkey sub-org and HD wallet
///
/// Creates a Turnkey sub-organization, an HD wallet within it, and optionally
/// a root user (for person entities) or a delegated signing account (for org entities).
/// Stores the result in the database.
#[utoipa::path(
    post,
    path = "/internal/entity-wallet",
    request_body = CreateEntityWallet,
    responses(
        (status = 201, description = "Entity wallet created successfully", body = EntityWalletResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entity_wallets"
)]
pub async fn create_entity_wallet(
    State(state): State<AppState>,
    Json(create): Json<CreateEntityWallet>,
) -> Result<impl IntoResponse, ApiError> {
    // Build root users (for person entities with passkeys/API keys)
    let root_users = build_root_users(&create)?;

    // Step 1: Create Turnkey sub-org
    let sub_org_result = state
        .turnkey
        .create_sub_organization(
            state.turnkey_parent_org_id.clone(),
            state.turnkey.current_timestamp(),
            CreateSubOrganizationIntentV7 {
                sub_organization_name: create.sub_org_name,
                root_users,
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

    let sub_org_id = &sub_org_result.result.sub_organization_id;

    // Step 2: Create HD wallet in the sub-org
    let wallet_result = state
        .turnkey
        .create_wallet(
            sub_org_id.clone(),
            state.turnkey.current_timestamp(),
            CreateWalletIntent {
                wallet_name: format!("entity-wallet-{}", create.entity_id),
                accounts: vec![WalletAccountParams {
                    curve: Curve::Secp256k1,
                    path_format: PathFormat::Bip32,
                    path: ETH_DERIVATION_PATH.to_string(),
                    address_format: AddressFormat::Ethereum,
                }],
                mnemonic_length: None,
            },
        )
        .await?;

    let turnkey_wallet_id = &wallet_result.result.wallet_id;

    let wallet_address =
        ChecksumAddress::new(wallet_result.result.addresses.first().ok_or_else(|| {
            ApiError::Internal("no wallet address returned from turnkey".to_string())
        })?)
        .map_err(|e| ApiError::Internal(format!("invalid wallet address from turnkey: {e}")))?;

    // Step 3: Optionally create a delegated account (for org entities)
    let delegated_user_id = match (create.delegated_user_name, create.delegated_api_public_key) {
        (Some(user_name), Some(public_key)) => {
            let delegated_result = state
                .turnkey
                .create_users(
                    sub_org_id.clone(),
                    state.turnkey.current_timestamp(),
                    CreateUsersIntentV3 {
                        users: vec![UserParamsV3 {
                            user_name,
                            user_email: None,
                            user_phone_number: None,
                            api_keys: vec![ApiKeyParamsV2 {
                                api_key_name: "delegated-key".to_string(),
                                public_key,
                                curve_type: ApiKeyCurve::P256,
                                expiration_seconds: None,
                            }],
                            authenticators: vec![],
                            oauth_providers: vec![],
                            user_tags: vec![],
                        }],
                    },
                )
                .await?;

            let user_id = delegated_result
                .result
                .user_ids
                .into_iter()
                .next()
                .ok_or_else(|| {
                    ApiError::Internal("no delegated user ID returned from turnkey".to_string())
                })?;

            // Create signing-only policy for the delegated account
            let condition = format!(
                "turnkey.activity.type == '{}' && turnkey.activity.resource.userId == '{user_id}'",
                "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2"
            );

            state
                .turnkey
                .create_policy(
                    sub_org_id.clone(),
                    state.turnkey.current_timestamp(),
                    CreatePolicyIntentV3 {
                        policy_name: format!("signing-only-{user_id}"),
                        effect: Effect::Allow,
                        condition: Some(condition),
                        consensus: None,
                        notes: "restrict delegated account to sign_raw_payload only".to_string(),
                    },
                )
                .await?;

            Some(user_id)
        }
        _ => None,
    };

    // Step 4: Store in database
    let turnkey_sub_org_id = TurnkeySubOrgId::new(sub_org_id)
        .map_err(|e| ApiError::Internal(format!("invalid sub-org ID from turnkey: {e}")))?;

    let repo = EntityWalletRepositoryImpl::new(&state.db);
    let entity_wallet = repo
        .create(
            create.entity_id,
            &turnkey_sub_org_id,
            turnkey_wallet_id,
            &wallet_address,
            delegated_user_id.as_deref(),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(EntityWalletResponse::from(entity_wallet)),
    ))
}

/// Look up an entity wallet by entity ID (internal endpoint)
#[utoipa::path(
    get,
    path = "/internal/entity-wallet/{entity_id}",
    params(
        ("entity_id" = Uuid, Path, description = "Entity ID (person or org)")
    ),
    responses(
        (status = 200, description = "Entity wallet found", body = EntityWalletResponse),
        (status = 404, description = "Entity wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entity_wallets"
)]
pub async fn get_entity_wallet(
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<EntityWalletResponse>, ApiError> {
    let repo = EntityWalletRepositoryImpl::new(&state.db);
    let wallet = repo
        .get_by_entity_id(entity_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(wallet.into()))
}

/// Look up an entity wallet by on-chain wallet address (internal endpoint)
#[utoipa::path(
    get,
    path = "/internal/entity-wallet-by-address/{address}",
    params(
        ("address" = String, Path, description = "EIP-55 checksummed wallet address")
    ),
    responses(
        (status = 200, description = "Entity wallet found", body = EntityWalletResponse),
        (status = 404, description = "No wallet found for this address"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entity_wallets"
)]
pub async fn get_entity_wallet_by_address(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<EntityWalletResponse>, ApiError> {
    let repo = EntityWalletRepositoryImpl::new(&state.db);
    let wallet = repo
        .get_by_wallet_address(&address)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(wallet.into()))
}

/// Build the root users list for Turnkey sub-org creation.
///
/// For person entities, this includes passkey authenticators and/or API keys.
/// For org entities, this returns an empty list (no root users).
fn build_root_users(create: &CreateEntityWallet) -> Result<Vec<RootUserParamsV4>, ApiError> {
    let root_user_name = match &create.root_user_name {
        Some(name) => name,
        None => return Ok(vec![]),
    };

    let api_keys: Vec<ApiKeyParamsV2> = match (&create.api_key_name, &create.api_public_key) {
        (Some(name), Some(key)) => vec![ApiKeyParamsV2 {
            api_key_name: name.clone(),
            public_key: key.clone(),
            curve_type: ApiKeyCurve::P256,
            expiration_seconds: None,
        }],
        _ => vec![],
    };

    let authenticators: Vec<AuthenticatorParamsV2> = create
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

    if api_keys.is_empty() && authenticators.is_empty() {
        return Err(ApiError::BadRequest(
            "root_user_name provided but no API key credentials or passkey authenticators given"
                .to_string(),
        ));
    }

    Ok(vec![RootUserParamsV4 {
        user_name: root_user_name.clone(),
        user_email: None,
        user_phone_number: None,
        api_keys,
        authenticators,
        oauth_providers: vec![],
    }])
}
