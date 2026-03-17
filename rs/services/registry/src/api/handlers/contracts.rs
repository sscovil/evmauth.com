use std::time::Duration;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use pagination::{Page, PaginatedResponse};
use serde::Deserialize;
use uuid::Uuid;

const WALLETS_SERVICE_TIMEOUT: Duration = Duration::from_secs(30);

use types::ChecksumAddress;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::{CreateContract, CreateRoleGrant};
use crate::dto::response::{ContractResponse, RoleGrantResponse, SendTxResponse};
use crate::repository::contract::{
    ContractRepository, ContractRepositoryImpl, CreateContractParams,
};
use crate::repository::role_grant::{RoleGrantRepository, RoleGrantRepositoryImpl};

/// Response from wallets internal entity-app-wallet endpoint.
/// Used to look up the org's app-specific wallet for use as EVMAuth default admin.
#[derive(Debug, Deserialize)]
struct EntityAppWalletResponse {
    wallet_address: ChecksumAddress,
}

/// Request body for wallets internal send-tx endpoint
#[derive(Debug, serde::Serialize)]
struct SendTxRequest {
    entity_id: Uuid,
    to: Option<ChecksumAddress>,
    calldata: String,
}

/// Send a transaction via the wallets service with a timeout.
async fn send_tx_via_wallets(
    state: &AppState,
    request: &SendTxRequest,
) -> Result<SendTxResponse, ApiError> {
    let wallets_url = &state.config.wallets_service_url;
    tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
        state
            .http_client
            .post(format!("{wallets_url}/internal/transactions"))
            .json(request)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("failed to reach wallets service: {e}")))?
            .error_for_status()
            .map_err(|e| ApiError::Internal(format!("transaction broadcast failed: {e}")))?
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("failed to parse send-tx response: {e}")))
    })
    .await
    .map_err(|_| ApiError::Internal("wallets service request timed out".to_string()))?
}

#[utoipa::path(
    post,
    path = "/orgs/{org_id}/contracts",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateContract,
    responses(
        (status = 201, description = "Contract deployment initiated", body = ContractResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn deploy_contract(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<CreateContract>,
) -> Result<impl IntoResponse, ApiError> {
    let wallets_url = &state.config.wallets_service_url;

    // Contract deployment requires an app registration to derive the admin wallet
    let app_registration_id = body.app_registration_id.ok_or_else(|| {
        ApiError::BadRequest("app_registration_id is required for contract deployment".to_string())
    })?;

    // Step 1: Look up the org's entity app wallet for this app registration
    // This derived address becomes the EVMAuth default admin for the proxy contract
    let app_wallet: EntityAppWalletResponse =
        tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
            state
                .http_client
                .get(format!(
                    "{wallets_url}/internal/entity-app-wallet/{org_id}/{app_registration_id}",
                ))
                .send()
                .await
                .map_err(|e| ApiError::Internal(format!("failed to reach wallets service: {e}")))?
                .error_for_status()
                .map_err(|e| {
                    if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                        ApiError::BadRequest(format!(
                            "no app wallet found for organization {org_id} and app {app_registration_id}",
                        ))
                    } else {
                        ApiError::Internal(format!("wallets service error: {e}"))
                    }
                })?
                .json()
                .await
                .map_err(|e| {
                    ApiError::Internal(format!("failed to parse wallets response: {e}"))
                })
        })
        .await
        .map_err(|_| ApiError::Internal("wallets service request timed out".to_string()))??;

    // Step 2: Encode BeaconProxy deployment bytecode with initialize() calldata
    let beacon_address = state.evm.platform_contract_address();
    let platform_operator = state.evm.platform_operator_address();

    // Parse the org's app-specific wallet address for use as initialDefaultAdmin and initialTreasury
    let app_wallet_addr: evm::Address = app_wallet
        .wallet_address
        .as_str()
        .parse()
        .map_err(|e| ApiError::Internal(format!("invalid app wallet address: {e}")))?;

    let init_data =
        evm::EvmClient::encode_initialize(app_wallet_addr, app_wallet_addr, platform_operator, "");

    let deploy_bytecode = evm::encode_beacon_proxy_deploy(beacon_address, init_data)?;
    let calldata_hex = format!("0x{}", alloy::hex::encode(&deploy_bytecode));

    // Step 3: Send deployment transaction via wallets service
    let send_tx_response = send_tx_via_wallets(
        &state,
        &SendTxRequest {
            entity_id: org_id,
            to: None,
            calldata: calldata_hex,
        },
    )
    .await?;

    let contract_address = send_tx_response.contract_address.ok_or_else(|| {
        ApiError::Internal("no contract address returned from deployment".to_string())
    })?;

    // Step 4: Insert contract record
    let repo = ContractRepositoryImpl::new(&state.db);
    let chain_id = state.evm.config().chain_id.to_string();
    let beacon_checksum = ChecksumAddress::new(&format!("{:#x}", beacon_address))
        .map_err(|e| ApiError::Internal(format!("invalid beacon address: {e}")))?;

    let contract = repo
        .create(CreateContractParams {
            org_id,
            app_registration_id: Some(app_registration_id),
            name: body.name,
            address: contract_address,
            chain_id,
            beacon_address: beacon_checksum,
            deploy_tx_hash: send_tx_response.tx_hash,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(ContractResponse::from(contract))))
}

#[utoipa::path(
    get,
    path = "/orgs/{org_id}/contracts",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("first" = Option<i64>, Query, description = "Number of items (forward)"),
        ("after" = Option<String>, Query, description = "Cursor (forward)"),
        ("last" = Option<i64>, Query, description = "Number of items (backward)"),
        ("before" = Option<String>, Query, description = "Cursor (backward)")
    ),
    responses(
        (status = 200, description = "List of contracts", body = PaginatedResponse<ContractResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn list_contracts(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(page): Query<Page>,
) -> Result<Json<PaginatedResponse<ContractResponse>>, ApiError> {
    let repo = ContractRepositoryImpl::new(&state.db);
    let results = repo.list_by_org_id(org_id, &page).await?;

    let responses: Vec<ContractResponse> = results.into_iter().map(Into::into).collect();
    Ok(Json(PaginatedResponse::from_page(responses, &page)))
}

#[utoipa::path(
    get,
    path = "/orgs/{org_id}/contracts/{contract_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    responses(
        (status = 200, description = "Contract found", body = ContractResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn get_contract(
    State(state): State<AppState>,
    Path((org_id, contract_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ContractResponse>, ApiError> {
    let repo = ContractRepositoryImpl::new(&state.db);
    let contract = repo.get(contract_id).await?.ok_or(ApiError::NotFound)?;

    if contract.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    Ok(Json(contract.into()))
}

#[utoipa::path(
    post,
    path = "/orgs/{org_id}/contracts/{contract_id}/roles",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    request_body = CreateRoleGrant,
    responses(
        (status = 201, description = "Role grant created", body = RoleGrantResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Contract not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn create_role_grant(
    State(state): State<AppState>,
    Path((org_id, contract_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreateRoleGrant>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate role name
    let role_bytes = evm::roles::role_name_to_bytes(&body.role).ok_or_else(|| {
        ApiError::BadRequest(format!(
            "invalid role name '{}'; valid roles: {}",
            body.role,
            evm::roles::VALID_ROLE_NAMES.join(", ")
        ))
    })?;

    let contract_repo = ContractRepositoryImpl::new(&state.db);
    let contract = contract_repo
        .get(contract_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if contract.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    // Check for existing active grant of the same role
    let grant_repo = RoleGrantRepositoryImpl::new(&state.db);
    if let Some(existing) = grant_repo
        .get_active_by_contract_and_role(contract_id, &body.role)
        .await?
        && existing.active
    {
        return Err(ApiError::BadRequest(format!(
            "contract already has an active grant for role {}",
            body.role
        )));
    }

    // Encode grantRole(role, platformOperator) calldata
    let platform_operator = state.evm.platform_operator_address();
    let calldata = evm::EvmClient::encode_grant_role(role_bytes, platform_operator);
    let calldata_hex = format!("0x{}", alloy::hex::encode(&calldata));

    // Send transaction via wallets service
    let send_tx_response = send_tx_via_wallets(
        &state,
        &SendTxRequest {
            entity_id: org_id,
            to: Some(contract.address.clone()),
            calldata: calldata_hex,
        },
    )
    .await?;

    // Record the grant
    let grant = grant_repo
        .create(contract_id, &body.role, &send_tx_response.tx_hash)
        .await?;

    Ok((StatusCode::CREATED, Json(RoleGrantResponse::from(grant))))
}

#[utoipa::path(
    delete,
    path = "/orgs/{org_id}/contracts/{contract_id}/roles/{role_grant_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID"),
        ("role_grant_id" = Uuid, Path, description = "Role grant ID")
    ),
    responses(
        (status = 200, description = "Role grant revoked", body = RoleGrantResponse),
        (status = 400, description = "No active grant to revoke"),
        (status = 404, description = "Contract not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn delete_role_grant(
    State(state): State<AppState>,
    Path((org_id, contract_id, role_grant_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<RoleGrantResponse>, ApiError> {
    let contract_repo = ContractRepositoryImpl::new(&state.db);
    let contract = contract_repo
        .get(contract_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if contract.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    let grant_repo = RoleGrantRepositoryImpl::new(&state.db);

    // Look up the grant to get the role name for encoding
    let grants = grant_repo.list_by_contract_id(contract_id).await?;
    let existing = grants
        .into_iter()
        .find(|g| g.id == role_grant_id)
        .ok_or_else(|| ApiError::BadRequest("role grant not found".to_string()))?;

    if !existing.active {
        return Err(ApiError::BadRequest(
            "role grant is already revoked".to_string(),
        ));
    }

    let role_bytes = evm::roles::role_name_to_bytes(&existing.role).ok_or_else(|| {
        ApiError::Internal(format!(
            "stored role name '{}' is not recognized",
            existing.role
        ))
    })?;

    // Encode revokeRole(role, platformOperator) calldata
    let platform_operator = state.evm.platform_operator_address();
    let calldata = evm::EvmClient::encode_revoke_role(role_bytes, platform_operator);
    let calldata_hex = format!("0x{}", alloy::hex::encode(&calldata));

    // Send transaction via wallets service
    let send_tx_response = send_tx_via_wallets(
        &state,
        &SendTxRequest {
            entity_id: org_id,
            to: Some(contract.address.clone()),
            calldata: calldata_hex,
        },
    )
    .await?;

    // Update the grant record
    let revoked = grant_repo
        .revoke(existing.id, &send_tx_response.tx_hash)
        .await?;

    Ok(Json(RoleGrantResponse::from(revoked)))
}

#[utoipa::path(
    get,
    path = "/orgs/{org_id}/contracts/{contract_id}/roles",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    responses(
        (status = 200, description = "List of role grants", body = Vec<RoleGrantResponse>),
        (status = 404, description = "Contract not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn list_role_grants(
    State(state): State<AppState>,
    Path((org_id, contract_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<RoleGrantResponse>>, ApiError> {
    let contract_repo = ContractRepositoryImpl::new(&state.db);
    let contract = contract_repo
        .get(contract_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if contract.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    let grant_repo = RoleGrantRepositoryImpl::new(&state.db);
    let grants = grant_repo.list_by_contract_id(contract_id).await?;
    let responses: Vec<RoleGrantResponse> = grants.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}
