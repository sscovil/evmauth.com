use alloy::consensus::{SignableTransaction, TxEip1559};
use alloy::eips::eip2718::Typed2718;
use alloy::hex;
use alloy::primitives::{Address, Bytes, Signature, TxKind, U256};
use alloy::providers::Provider;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::repository::org_wallet::{OrgWalletRepository, OrgWalletRepositoryImpl};

use turnkey::signing::{HashFunction, PayloadEncoding, SignRawPayloadParams};

/// Request to sign and broadcast a transaction via a delegated account
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendTxRequest {
    /// The organization ID whose delegated account should sign
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Uuid,

    /// Target contract address (None = contract creation)
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub to: Option<String>,

    /// Hex-encoded calldata or deploy bytecode
    #[schema(example = "0xdeadbeef")]
    pub calldata: String,
}

/// Response containing the transaction hash and optional contract address
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendTxResponse {
    /// The transaction hash
    #[schema(example = "0xabc123...")]
    pub tx_hash: String,

    /// The deployed contract address (populated for contract creation)
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub contract_address: Option<String>,
}

/// Sign and broadcast a transaction via a delegated account
///
/// Looks up the org's delegated account, signs the transaction via Turnkey,
/// and broadcasts it to the chain. For contract creation transactions (to = None),
/// returns the computed contract address.
#[utoipa::path(
    post,
    path = "/internal/send-tx",
    request_body = SendTxRequest,
    responses(
        (status = 200, description = "Transaction broadcast successfully", body = SendTxResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Org wallet or delegated account not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/send_tx"
)]
pub async fn send_tx(
    State(state): State<AppState>,
    Json(request): Json<SendTxRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Look up the org wallet
    let repo = OrgWalletRepositoryImpl::new(&state.db);
    let org_wallet = repo
        .get_by_org_id(request.org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let delegated_user_id = org_wallet.turnkey_delegated_user_id.ok_or_else(|| {
        ApiError::BadRequest(format!(
            "Organization {} does not have a delegated signing account",
            request.org_id,
        ))
    })?;

    let from_address: Address = org_wallet
        .wallet_address
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid wallet address in database: {e}")))?;

    // Step 2: Parse calldata
    let calldata_hex = request
        .calldata
        .strip_prefix("0x")
        .unwrap_or(&request.calldata);
    let calldata = hex::decode(calldata_hex)
        .map_err(|e| ApiError::BadRequest(format!("Invalid calldata hex: {e}")))?;

    // Step 3: Determine tx kind (create vs call)
    let tx_kind = match &request.to {
        Some(addr) => {
            let to: Address = addr
                .parse()
                .map_err(|e| ApiError::BadRequest(format!("Invalid target address: {e}")))?;
            TxKind::Call(to)
        }
        None => TxKind::Create,
    };

    // Step 4: Get nonce and gas estimates from the chain
    let provider = state.chain.provider();

    let nonce = provider
        .get_transaction_count(from_address)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get nonce: {e}")))?;

    let chain_id = state.chain.config().chain_id;
    let input_bytes = Bytes::from(calldata);

    // Build unsigned transaction
    let mut tx = TxEip1559 {
        chain_id,
        nonce,
        gas_limit: 0,
        max_fee_per_gas: 0,
        max_priority_fee_per_gas: 0,
        to: tx_kind,
        value: U256::ZERO,
        input: input_bytes.clone(),
        access_list: Default::default(),
    };

    // Build estimate gas request
    let mut gas_req = alloy::rpc::types::TransactionRequest::default()
        .from(from_address)
        .input(alloy::rpc::types::TransactionInput::new(input_bytes));

    if let TxKind::Call(addr) = tx_kind {
        gas_req = gas_req.to(addr);
    }

    let gas_estimate = provider
        .estimate_gas(gas_req)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to estimate gas: {e}")))?;

    let gas_price = provider
        .get_gas_price()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get gas price: {e}")))?;

    tx.gas_limit = (gas_estimate * 120) / 100; // 20% buffer
    tx.max_fee_per_gas = gas_price * 2; // 2x current price for safety
    tx.max_priority_fee_per_gas = 1_000_000_000; // 1 gwei tip

    // Step 5: Sign the transaction hash via Turnkey
    let sig_hash = tx.signature_hash();
    let sig_hash_hex = hex::encode(sig_hash.as_slice());

    let signature = state
        .turnkey
        .sign_raw_payload(SignRawPayloadParams {
            sub_org_id: org_wallet.turnkey_sub_org_id,
            user_id: delegated_user_id,
            payload: sig_hash_hex,
            encoding: PayloadEncoding::PayloadEncodingHexadecimal,
            hash_function: HashFunction::HashFunctionNoOp,
        })
        .await?;

    // Step 6: Reconstruct the signature
    let r = U256::from_str_radix(signature.r.strip_prefix("0x").unwrap_or(&signature.r), 16)
        .map_err(|e| ApiError::Internal(format!("Invalid signature r: {e}")))?;

    let s = U256::from_str_radix(signature.s.strip_prefix("0x").unwrap_or(&signature.s), 16)
        .map_err(|e| ApiError::Internal(format!("Invalid signature s: {e}")))?;

    let v: bool = signature
        .v
        .strip_prefix("0x")
        .unwrap_or(&signature.v)
        .parse::<u64>()
        .map(|val| val == 1 || val == 28)
        .map_err(|e| ApiError::Internal(format!("Invalid signature v: {e}")))?;

    let sig = Signature::new(r, s, v);

    // Step 7: Encode signed transaction and broadcast
    let signed_tx = tx.into_signed(sig);
    let mut encoded = Vec::new();
    // EIP-2718 type envelope: type prefix + RLP
    encoded.push(signed_tx.tx().ty());
    signed_tx.rlp_encode(&mut encoded);

    let pending = provider
        .send_raw_transaction(&encoded)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to broadcast transaction: {e}")))?;

    let tx_hash = format!("{:#x}", pending.tx_hash());

    // Step 8: Compute contract address for deployments
    let contract_address = if request.to.is_none() {
        Some(format!("{:#x}", from_address.create(nonce)))
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(SendTxResponse {
            tx_hash,
            contract_address,
        }),
    ))
}
