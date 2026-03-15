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
use crate::dto::request::CreatePersonAppWallet;
use crate::dto::response::PersonAppWalletResponse;
use crate::repository::person_app_wallet::{
    PersonAppWalletRepository, PersonAppWalletRepositoryImpl,
};
use crate::repository::person_turnkey_ref::{
    PersonTurnkeyRefRepository, PersonTurnkeyRefRepositoryImpl,
};

/// Create an HD wallet account for a (person, app) pair
///
/// Looks up the person's Turnkey sub-org, creates a new wallet account,
/// and stores the mapping in the database.
#[utoipa::path(
    post,
    path = "/internal/person-app-wallet",
    request_body = CreatePersonAppWallet,
    responses(
        (status = 201, description = "Person app wallet created successfully", body = PersonAppWalletResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Person sub-org not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/person_app_wallets"
)]
pub async fn create_person_app_wallet(
    State(state): State<AppState>,
    Json(create): Json<CreatePersonAppWallet>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Look up the person's Turnkey sub-org
    let ref_repo = PersonTurnkeyRefRepositoryImpl::new(&state.db);
    let turnkey_ref = ref_repo
        .get_by_person_id(create.person_id)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "person {} does not have a Turnkey sub-org",
                create.person_id,
            ))
        })?;

    // Step 2: Create a wallet account in the person's sub-org
    let account_result = state
        .turnkey
        .create_wallet_accounts(
            turnkey_ref.turnkey_sub_org_id.to_string(),
            state.turnkey.current_timestamp(),
            CreateWalletAccountsIntent {
                wallet_id: format!("wallet-{}", turnkey_ref.turnkey_sub_org_id),
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
    let wallet_repo = PersonAppWalletRepositoryImpl::new(&state.db);
    let app_wallet = wallet_repo
        .create(
            create.person_id,
            create.app_registration_id,
            &wallet_address,
            "", // account_id not returned by create_wallet_accounts
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(PersonAppWalletResponse::from(app_wallet)),
    ))
}

/// Look up a person app wallet by ID (internal endpoint)
#[utoipa::path(
    get,
    path = "/internal/person-app-wallet/{id}",
    params(
        ("id" = Uuid, Path, description = "Person app wallet ID")
    ),
    responses(
        (status = 200, description = "Person app wallet found", body = PersonAppWalletResponse),
        (status = 404, description = "Person app wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/person_app_wallets"
)]
pub async fn get_person_app_wallet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PersonAppWalletResponse>, ApiError> {
    let repo = PersonAppWalletRepositoryImpl::new(&state.db);
    let wallet = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(wallet.into()))
}
