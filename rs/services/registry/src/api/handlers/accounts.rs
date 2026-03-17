use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;
use crate::api::error::ApiError;

/// Query parameters for the accounts endpoint.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AccountsQuery {
    /// Contract address to query balances on
    pub contract: String,
    /// Optional delegate address to verify as operator
    pub delegate: Option<String>,
}

/// A single token balance entry.
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenBalance {
    /// Token ID
    #[schema(example = "1")]
    pub id: String,
    /// Balance as decimal string
    #[schema(example = "5")]
    pub balance: String,
}

/// Response from the accounts endpoint.
#[derive(Debug, Serialize, ToSchema)]
pub struct AccountsResponse {
    /// Queried account address
    #[schema(example = "0x1234...abcd")]
    pub address: String,
    /// Contract address queried
    #[schema(example = "0x5678...ef01")]
    pub contract: String,
    /// Chain ID
    #[schema(example = "31337")]
    pub chain_id: String,
    /// Chain name
    #[schema(example = "localhost")]
    pub chain_name: String,
    /// Delegate address (if queried)
    pub delegate: Option<String>,
    /// Token balances
    pub tokens: Vec<TokenBalance>,
    /// ISO 8601 timestamp of the query
    #[schema(example = "2026-03-17T12:00:00Z")]
    pub queried_at: String,
}

/// Query on-chain token balances for an account.
///
/// Public endpoint -- reads live on-chain state from the specified EVMAuth6909
/// contract. Optionally verifies a delegate address as an operator.
#[utoipa::path(
    get,
    path = "/accounts/{address}",
    params(
        ("address" = String, Path, description = "Account address to query"),
        ("contract" = String, Query, description = "Contract address"),
        ("delegate" = Option<String>, Query, description = "Optional delegate to verify")
    ),
    responses(
        (status = 200, description = "Account balances", body = AccountsResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Delegate is not an operator"),
        (status = 500, description = "Internal server error")
    ),
    tag = "accounts"
)]
pub async fn get_account(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(params): Query<AccountsQuery>,
) -> Result<Json<AccountsResponse>, ApiError> {
    let account: evm::Address = address
        .parse()
        .map_err(|e| ApiError::BadRequest(format!("invalid account address: {e}")))?;

    let contract: evm::Address = params
        .contract
        .parse()
        .map_err(|e| ApiError::BadRequest(format!("invalid contract address: {e}")))?;

    // If a delegate is specified, verify they are an operator for the account
    if let Some(ref delegate_str) = params.delegate {
        let delegate: evm::Address = delegate_str
            .parse()
            .map_err(|e| ApiError::BadRequest(format!("invalid delegate address: {e}")))?;

        let is_op = state
            .evm
            .is_operator_on_contract(contract, account, delegate)
            .await
            .map_err(|e| {
                tracing::warn!("is_operator check failed: {e}");
                ApiError::Internal("on-chain operator check failed".to_string())
            })?;

        if !is_op {
            return Err(ApiError::BadRequest(
                "delegate is not an operator for this account".to_string(),
            ));
        }
    }

    // Query a standard set of token IDs (1-10) which covers typical usage.
    // TODO: accept token_ids as a query parameter or look up relevant_token_ids
    // from the app registration via client_id.
    let token_ids: Vec<evm::U256> = (1u64..=10).map(evm::U256::from).collect();

    let balances = state
        .evm
        .balances_for(contract, account, &token_ids)
        .await
        .map_err(|e| {
            tracing::warn!("balances_for query failed: {e}");
            ApiError::Internal("on-chain balance query failed".to_string())
        })?;

    let tokens: Vec<TokenBalance> = balances
        .into_iter()
        .filter(|(_, balance)| !balance.is_zero())
        .map(|(id, balance)| TokenBalance {
            id: id.to_string(),
            balance: balance.to_string(),
        })
        .collect();

    let chain_id = state.evm.config().chain_id;
    let chain_name = match chain_id {
        1 => "mainnet",
        11155111 => "sepolia",
        31337 => "localhost",
        _ => "unknown",
    };

    Ok(Json(AccountsResponse {
        address: format!("{account}"),
        contract: format!("{contract}"),
        chain_id: chain_id.to_string(),
        chain_name: chain_name.to_string(),
        delegate: params.delegate,
        tokens,
        queried_at: chrono::Utc::now().to_rfc3339(),
    }))
}
