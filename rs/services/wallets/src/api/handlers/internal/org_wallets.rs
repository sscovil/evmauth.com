use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use turnkey_client::generated::immutable::activity::api::ApiKeyParams as SdkApiKeyParams;
use turnkey_client::generated::immutable::activity::v1::{
    ApiKeyParamsV2, CreateApiOnlyUsersIntent, CreateSubOrganizationIntentV7, CreateWalletIntent,
    WalletAccountParams,
};
use turnkey_client::generated::immutable::common::v1::{
    AddressFormat, ApiKeyCurve, Curve, PathFormat,
};

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::CreateOrgWallet;
use crate::dto::response::OrgWalletResponse;
use crate::repository::org_wallet::{OrgWalletRepository, OrgWalletRepositoryImpl};

/// Create an org wallet with Turnkey sub-org and optional delegated account
///
/// Creates a Turnkey sub-organization, a wallet within it, and optionally
/// a delegated signing account. Stores the result in the database.
#[utoipa::path(
    post,
    path = "/internal/org-wallet",
    request_body = CreateOrgWallet,
    responses(
        (status = 201, description = "Org wallet created successfully", body = OrgWalletResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/org_wallets"
)]
pub async fn create_org_wallet(
    State(state): State<AppState>,
    Json(create): Json<CreateOrgWallet>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Create Turnkey sub-org
    let sub_org_result = state
        .turnkey
        .create_sub_organization(
            state.turnkey_parent_org_id.clone(),
            state.turnkey.current_timestamp(),
            CreateSubOrganizationIntentV7 {
                sub_organization_name: create.sub_org_name,
                root_users: vec![],
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

    // Step 2: Create wallet in the sub-org
    let wallet_result = state
        .turnkey
        .create_wallet(
            sub_org_id.clone(),
            state.turnkey.current_timestamp(),
            CreateWalletIntent {
                wallet_name: format!("org-wallet-{}", create.org_id),
                accounts: vec![WalletAccountParams {
                    curve: Curve::Secp256k1,
                    path_format: PathFormat::Bip32,
                    path: "m/44'/60'/0'/0/0".to_string(),
                    address_format: AddressFormat::Ethereum,
                }],
                mnemonic_length: None,
            },
        )
        .await?;

    let wallet_address = wallet_result
        .result
        .addresses
        .first()
        .ok_or_else(|| ApiError::Internal("no wallet address returned from turnkey".to_string()))?
        .clone();

    // Step 3: Optionally create a delegated account
    let delegated_user_id = match (create.delegated_user_name, create.delegated_api_public_key) {
        (Some(user_name), Some(public_key)) => {
            let delegated_result = state
                .turnkey
                .create_api_only_users(
                    sub_org_id.clone(),
                    state.turnkey.current_timestamp(),
                    CreateApiOnlyUsersIntent {
                        api_only_users: vec![
                            turnkey_client::generated::immutable::activity::v1::ApiOnlyUserParams {
                                user_name,
                                user_email: None,
                                user_tags: vec!["delegated-signing".to_string()],
                                api_keys: vec![SdkApiKeyParams {
                                    api_key_name: "delegated-key".to_string(),
                                    public_key,
                                    expiration_seconds: None,
                                }],
                            },
                        ],
                    },
                )
                .await?;

            delegated_result.result.user_ids.into_iter().next()
        }
        _ => None,
    };

    // Step 4: Store in database
    let repo = OrgWalletRepositoryImpl::new(&state.db);
    let org_wallet = repo
        .create(
            create.org_id,
            sub_org_id,
            &wallet_address,
            delegated_user_id.as_deref(),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(OrgWalletResponse::from(org_wallet)),
    ))
}

/// Look up an org wallet by organization ID (internal endpoint)
#[utoipa::path(
    get,
    path = "/internal/org-wallet/{org_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Org wallet found", body = OrgWalletResponse),
        (status = 404, description = "Org wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/org_wallets"
)]
pub async fn get_org_wallet_internal(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrgWalletResponse>, ApiError> {
    let repo = OrgWalletRepositoryImpl::new(&state.db);
    let wallet = repo
        .get_by_org_id(org_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(wallet.into()))
}
