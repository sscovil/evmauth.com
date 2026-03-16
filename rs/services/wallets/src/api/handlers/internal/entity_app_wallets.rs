use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use turnkey_client::generated::immutable::activity::v1::{
    CreateWalletAccountsIntent, WalletAccountParams,
};
use turnkey_client::generated::immutable::common::v1::{AddressFormat, Curve, PathFormat};

use types::ChecksumAddress;

use crate::AppState;

/// BIP-32 derivation path for Ethereum accounts (EIP-44)
const ETH_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";
use crate::api::error::ApiError;
use crate::dto::request::CreateEntityAppWallet;
use crate::dto::response::EntityAppWalletResponse;
use crate::repository::entity_app_wallet::{
    EntityAppWalletRepository, EntityAppWalletRepositoryImpl,
};
use crate::repository::entity_wallet::{EntityWalletRepository, EntityWalletRepositoryImpl};

/// Create an HD wallet account for an (entity, app) pair
///
/// Looks up the entity's Turnkey sub-org and HD wallet, creates a new wallet
/// account, and stores the mapping in the database.
#[utoipa::path(
    post,
    path = "/internal/entity-app-wallet",
    request_body = CreateEntityAppWallet,
    responses(
        (status = 201, description = "Entity app wallet created successfully", body = EntityAppWalletResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Entity wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entity_app_wallets"
)]
pub async fn create_entity_app_wallet(
    State(state): State<AppState>,
    Json(create): Json<CreateEntityAppWallet>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Look up the entity's wallet (sub-org + HD wallet)
    let entity_repo = EntityWalletRepositoryImpl::new(&state.db);
    let entity_wallet = entity_repo
        .get_by_entity_id(create.entity_id)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "entity {} does not have a wallet",
                create.entity_id,
            ))
        })?;

    // Step 2: Create a new wallet account in the entity's sub-org
    let account_result = state
        .turnkey
        .create_wallet_accounts(
            entity_wallet.turnkey_sub_org_id.to_string(),
            state.turnkey.current_timestamp(),
            CreateWalletAccountsIntent {
                wallet_id: entity_wallet.turnkey_wallet_id.clone(),
                accounts: vec![WalletAccountParams {
                    curve: Curve::Secp256k1,
                    path_format: PathFormat::Bip32,
                    path: ETH_DERIVATION_PATH.to_string(),
                    address_format: AddressFormat::Ethereum,
                }],
                persist: None,
            },
        )
        .await?;

    let wallet_address =
        ChecksumAddress::new(
            account_result.result.addresses.first().ok_or_else(|| {
                ApiError::Internal("no address returned from turnkey".to_string())
            })?,
        )
        .map_err(|e| ApiError::Internal(format!("invalid wallet address from turnkey: {e}")))?;

    // Step 3: Store in database
    let app_wallet_repo = EntityAppWalletRepositoryImpl::new(&state.db);
    let app_wallet = app_wallet_repo
        .create(
            create.entity_id,
            create.app_registration_id,
            &wallet_address,
            "", // account_id not returned by create_wallet_accounts
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(EntityAppWalletResponse::from(app_wallet)),
    ))
}

/// Look up an entity app wallet by entity ID and app registration ID (internal endpoint)
#[utoipa::path(
    get,
    path = "/internal/entity-app-wallet/{entity_id}/{app_id}",
    params(
        ("entity_id" = Uuid, Path, description = "Entity ID (person or org)"),
        ("app_id" = Uuid, Path, description = "App registration ID")
    ),
    responses(
        (status = 200, description = "Entity app wallet found", body = EntityAppWalletResponse),
        (status = 404, description = "Entity app wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entity_app_wallets"
)]
pub async fn get_entity_app_wallet(
    State(state): State<AppState>,
    Path((entity_id, app_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<EntityAppWalletResponse>, ApiError> {
    let repo = EntityAppWalletRepositoryImpl::new(&state.db);
    let wallet = repo
        .get_by_entity_and_app(entity_id, app_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(wallet.into()))
}
