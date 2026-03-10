use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::CreateOrgWallet;
use crate::dto::response::OrgWalletResponse;
use crate::repository::org_wallet::{OrgWalletRepository, OrgWalletRepositoryImpl};

use turnkey::sub_org::{ApiKeyParams, CreateDelegatedAccount, CreateSubOrg, CreateWallet};

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
    let sub_org_response = state
        .turnkey
        .create_sub_org(CreateSubOrg {
            name: create.sub_org_name,
            root_users: vec![],
        })
        .await?;

    let sub_org_id = &sub_org_response.sub_organization_id;

    // Step 2: Create wallet in the sub-org
    let wallet_response = state
        .turnkey
        .create_wallet(CreateWallet {
            sub_org_id: sub_org_id.clone(),
            wallet_name: format!("org-wallet-{}", create.org_id),
            accounts: 1,
        })
        .await?;

    let wallet_address = wallet_response
        .addresses
        .first()
        .ok_or_else(|| ApiError::Internal("No wallet address returned from Turnkey".to_string()))?
        .clone();

    // Step 3: Optionally create a delegated account
    let delegated_user_id = match (create.delegated_user_name, create.delegated_api_public_key) {
        (Some(user_name), Some(public_key)) => {
            let delegated_response = state
                .turnkey
                .create_delegated_account(CreateDelegatedAccount {
                    sub_org_id: sub_org_id.clone(),
                    user_name,
                    api_key: ApiKeyParams {
                        api_key_name: "delegated-key".to_string(),
                        public_key,
                    },
                })
                .await?;

            Some(delegated_response.user_id)
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
