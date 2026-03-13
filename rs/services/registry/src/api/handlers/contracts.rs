use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use pagination::{Page, PaginatedResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::CreateContract;
use crate::dto::response::{ContractResponse, OperatorGrantResponse, SendTxResponse};
use crate::repository::contract::{
    ContractRepository, ContractRepositoryImpl, CreateContractParams,
};
use crate::repository::operator_grant::{OperatorGrantRepository, OperatorGrantRepositoryImpl};

/// Response from wallets internal org-wallet endpoint
#[derive(Debug, Deserialize)]
struct OrgWalletResponse {
    wallet_address: String,
}

/// Request body for wallets internal send-tx endpoint
#[derive(Debug, serde::Serialize)]
struct SendTxRequest {
    org_id: Uuid,
    to: Option<String>,
    calldata: String,
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

    // Step 1: Look up org wallet address
    let org_wallet: OrgWalletResponse = state
        .http_client
        .get(format!("{wallets_url}/internal/org-wallet/{org_id}"))
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to reach wallets service: {e}")))?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                ApiError::BadRequest(format!("No wallet found for organization {org_id}"))
            } else {
                ApiError::Internal(format!("Wallets service error: {e}"))
            }
        })?
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse wallets response: {e}")))?;

    // Step 2: Encode BeaconProxy deployment bytecode
    let beacon_address = state.evm.platform_contract_address();
    let deploy_bytecode = evm::encode_beacon_proxy_deploy(beacon_address, evm::Bytes::new());
    let calldata_hex = format!("0x{}", alloy::hex::encode(&deploy_bytecode));

    // Step 3: Send deployment transaction via wallets service
    let send_tx_response: SendTxResponse = state
        .http_client
        .post(format!("{wallets_url}/internal/send-tx"))
        .json(&SendTxRequest {
            org_id,
            to: None,
            calldata: calldata_hex,
        })
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to reach wallets service: {e}")))?
        .error_for_status()
        .map_err(|e| ApiError::Internal(format!("Transaction broadcast failed: {e}")))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse send-tx response: {e}")))?;

    let contract_address = send_tx_response.contract_address.ok_or_else(|| {
        ApiError::Internal("No contract address returned from deployment".to_string())
    })?;

    // Step 4: Insert contract record
    let repo = ContractRepositoryImpl::new(&state.db);
    let chain_id = state.evm.config().chain_id.to_string();

    let contract = repo
        .create(CreateContractParams {
            org_id,
            app_registration_id: body.app_registration_id,
            name: body.name,
            address: contract_address,
            chain_id,
            beacon_address: format!("{:#x}", beacon_address),
            deploy_tx_hash: send_tx_response.tx_hash,
        })
        .await?;

    let _ = org_wallet.wallet_address;

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
    path = "/orgs/{org_id}/contracts/{contract_id}/grant-operator",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    responses(
        (status = 201, description = "Operator grant created", body = OperatorGrantResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Contract not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn grant_operator(
    State(state): State<AppState>,
    Path((org_id, contract_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let contract_repo = ContractRepositoryImpl::new(&state.db);
    let contract = contract_repo
        .get(contract_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if contract.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    // Check for existing active grant
    let grant_repo = OperatorGrantRepositoryImpl::new(&state.db);
    if let Some(existing) = grant_repo.get_by_contract_id(contract_id).await?
        && existing.active
    {
        return Err(ApiError::BadRequest(
            "Contract already has an active operator grant".to_string(),
        ));
    }

    // Encode setOperator(platformOperator, true) calldata
    let platform_operator = state.evm.platform_contract_address();
    let calldata = evm::EvmClient::encode_set_operator(platform_operator, true);
    let calldata_hex = format!("0x{}", alloy::hex::encode(&calldata));

    // Send transaction via wallets service
    let wallets_url = &state.config.wallets_service_url;
    let send_tx_response: SendTxResponse = state
        .http_client
        .post(format!("{wallets_url}/internal/send-tx"))
        .json(&SendTxRequest {
            org_id,
            to: Some(contract.address),
            calldata: calldata_hex,
        })
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to reach wallets service: {e}")))?
        .error_for_status()
        .map_err(|e| ApiError::Internal(format!("Transaction broadcast failed: {e}")))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse send-tx response: {e}")))?;

    // Record the grant
    let grant = grant_repo
        .create(contract_id, &send_tx_response.tx_hash)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(OperatorGrantResponse::from(grant)),
    ))
}

#[utoipa::path(
    post,
    path = "/orgs/{org_id}/contracts/{contract_id}/revoke-operator",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    responses(
        (status = 200, description = "Operator grant revoked", body = OperatorGrantResponse),
        (status = 400, description = "No active grant to revoke"),
        (status = 404, description = "Contract not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "contracts"
)]
pub async fn revoke_operator(
    State(state): State<AppState>,
    Path((org_id, contract_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<OperatorGrantResponse>, ApiError> {
    let contract_repo = ContractRepositoryImpl::new(&state.db);
    let contract = contract_repo
        .get(contract_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if contract.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    let grant_repo = OperatorGrantRepositoryImpl::new(&state.db);
    let existing = grant_repo
        .get_by_contract_id(contract_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("No active operator grant to revoke".to_string()))?;

    if !existing.active {
        return Err(ApiError::BadRequest(
            "Operator grant is already revoked".to_string(),
        ));
    }

    // Encode setOperator(platformOperator, false) calldata
    let platform_operator = state.evm.platform_contract_address();
    let calldata = evm::EvmClient::encode_set_operator(platform_operator, false);
    let calldata_hex = format!("0x{}", alloy::hex::encode(&calldata));

    // Send transaction via wallets service
    let wallets_url = &state.config.wallets_service_url;
    let send_tx_response: SendTxResponse = state
        .http_client
        .post(format!("{wallets_url}/internal/send-tx"))
        .json(&SendTxRequest {
            org_id,
            to: Some(contract.address),
            calldata: calldata_hex,
        })
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to reach wallets service: {e}")))?
        .error_for_status()
        .map_err(|e| ApiError::Internal(format!("Transaction broadcast failed: {e}")))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse send-tx response: {e}")))?;

    // Update the grant record
    let revoked = grant_repo
        .revoke(existing.id, &send_tx_response.tx_hash)
        .await?;

    Ok(Json(OperatorGrantResponse::from(revoked)))
}
