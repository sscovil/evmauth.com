use std::time::Duration;

use alloy::consensus::{SignableTransaction, TxEip1559};
use alloy::eips::eip2718::Typed2718;
use alloy::hex;
use alloy::primitives::{Address, Bytes, Signature, TxKind, U256};
use alloy::providers::Provider;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use types::{ChecksumAddress, TxHash};

use crate::AppState;
use crate::api::error::ApiError;
use crate::repository::entity_wallet::{EntityWalletRepository, EntityWalletRepositoryImpl};

use turnkey_client::generated::immutable::activity::v1::SignRawPayloadIntentV2;
use turnkey_client::generated::immutable::common::v1::{HashFunction, PayloadEncoding};

/// Gas limit buffer: multiply estimate by 120/100 for 20% headroom
const GAS_LIMIT_BUFFER_NUMERATOR: u64 = 120;
const GAS_LIMIT_BUFFER_DENOMINATOR: u64 = 100;
/// Max fee per gas: multiply current gas price by this factor for safety
const MAX_FEE_GAS_PRICE_MULTIPLIER: u128 = 2;
/// Priority fee tip: 1 gwei
const PRIORITY_FEE_WEI: u128 = 1_000_000_000;
/// Timeout for individual RPC calls to the blockchain node
const RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Request to sign and broadcast a transaction via a delegated account
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendTxRequest {
    /// The entity ID whose delegated account should sign
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub entity_id: Uuid,

    /// Target contract address (None = contract creation)
    #[schema(example = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed")]
    pub to: Option<ChecksumAddress>,

    /// Hex-encoded calldata or deploy bytecode
    #[schema(example = "0xdeadbeef")]
    pub calldata: String,
}

/// Response containing the transaction hash and optional contract address
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendTxResponse {
    /// The transaction hash
    #[schema(example = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab")]
    pub tx_hash: TxHash,

    /// The deployed contract address (populated for contract creation)
    #[schema(example = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed")]
    pub contract_address: Option<ChecksumAddress>,
}

/// Sign and broadcast a transaction via a delegated account
///
/// Looks up the entity's delegated account, signs the transaction via Turnkey,
/// and broadcasts it to the chain. For contract creation transactions (to = None),
/// returns the computed contract address.
#[utoipa::path(
    post,
    path = "/internal/transactions",
    request_body = SendTxRequest,
    responses(
        (status = 200, description = "Transaction broadcast successfully", body = SendTxResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Entity wallet or delegated account not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/send_tx"
)]
pub async fn send_tx(
    State(state): State<AppState>,
    Json(request): Json<SendTxRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Look up the entity wallet
    let repo = EntityWalletRepositoryImpl::new(&state.db);
    let entity_wallet = repo
        .get_by_entity_id(request.entity_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let delegated_user_id = entity_wallet.turnkey_delegated_user_id.ok_or_else(|| {
        ApiError::BadRequest(format!(
            "entity {} does not have a delegated signing account",
            request.entity_id,
        ))
    })?;

    let from_address: Address = entity_wallet
        .wallet_address
        .as_str()
        .parse()
        .map_err(|e| ApiError::Internal(format!("invalid wallet address in database: {e}")))?;

    // Step 2: Parse calldata
    let calldata_hex = request
        .calldata
        .strip_prefix("0x")
        .unwrap_or(&request.calldata);
    let calldata = hex::decode(calldata_hex)
        .map_err(|e| ApiError::BadRequest(format!("invalid calldata hex: {e}")))?;

    // Step 3: Determine tx kind (create vs call)
    let tx_kind = match &request.to {
        Some(addr) => {
            let to: Address = addr
                .as_str()
                .parse()
                .map_err(|e| ApiError::BadRequest(format!("invalid target address: {e}")))?;
            TxKind::Call(to)
        }
        None => TxKind::Create,
    };

    // Step 4: Get nonce and gas estimates from the chain
    let provider = state.evm.provider();

    let nonce = tokio::time::timeout(RPC_TIMEOUT, provider.get_transaction_count(from_address))
        .await
        .map_err(|_| ApiError::Internal("nonce fetch timed out".to_string()))?
        .map_err(|e| ApiError::Internal(format!("failed to get nonce: {e}")))?;

    let chain_id = state.evm.config().chain_id;
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

    let gas_estimate = tokio::time::timeout(RPC_TIMEOUT, provider.estimate_gas(gas_req))
        .await
        .map_err(|_| ApiError::Internal("gas estimate timed out".to_string()))?
        .map_err(|e| ApiError::Internal(format!("failed to estimate gas: {e}")))?;

    let gas_price = tokio::time::timeout(RPC_TIMEOUT, provider.get_gas_price())
        .await
        .map_err(|_| ApiError::Internal("gas price fetch timed out".to_string()))?
        .map_err(|e| ApiError::Internal(format!("failed to get gas price: {e}")))?;

    tx.gas_limit = (gas_estimate * GAS_LIMIT_BUFFER_NUMERATOR) / GAS_LIMIT_BUFFER_DENOMINATOR;
    tx.max_fee_per_gas = gas_price * MAX_FEE_GAS_PRICE_MULTIPLIER;
    tx.max_priority_fee_per_gas = PRIORITY_FEE_WEI;

    // Step 5: Sign the transaction hash via Turnkey
    let sig_hash = tx.signature_hash();
    let sig_hash_hex = hex::encode(sig_hash.as_slice());

    let sign_result = state
        .turnkey
        .sign_raw_payload(
            entity_wallet.turnkey_sub_org_id.into_inner(),
            state.turnkey.current_timestamp(),
            SignRawPayloadIntentV2 {
                sign_with: delegated_user_id,
                payload: sig_hash_hex,
                encoding: PayloadEncoding::Hexadecimal,
                hash_function: HashFunction::NoOp,
            },
        )
        .await?;

    // Step 6: Reconstruct the signature
    let sig_r = &sign_result.result.r;
    let sig_s = &sign_result.result.s;
    let sig_v = &sign_result.result.v;

    let r = U256::from_str_radix(sig_r.strip_prefix("0x").unwrap_or(sig_r), 16)
        .map_err(|e| ApiError::Internal(format!("invalid signature r: {e}")))?;

    let s = U256::from_str_radix(sig_s.strip_prefix("0x").unwrap_or(sig_s), 16)
        .map_err(|e| ApiError::Internal(format!("invalid signature s: {e}")))?;

    let v: bool = sig_v
        .strip_prefix("0x")
        .unwrap_or(sig_v)
        .parse::<u64>()
        .map(|val| val == 1 || val == 28)
        .map_err(|e| ApiError::Internal(format!("invalid signature v: {e}")))?;

    let sig = Signature::new(r, s, v);

    // Step 7: Encode signed transaction and broadcast
    let signed_tx = tx.into_signed(sig);
    let mut encoded = Vec::new();
    // EIP-2718 type envelope: type prefix + RLP
    encoded.push(signed_tx.tx().ty());
    signed_tx.rlp_encode(&mut encoded);

    let pending = tokio::time::timeout(RPC_TIMEOUT, provider.send_raw_transaction(&encoded))
        .await
        .map_err(|_| ApiError::Internal("transaction broadcast timed out".to_string()))?
        .map_err(|e| ApiError::Internal(format!("failed to broadcast transaction: {e}")))?;

    let tx_hash = TxHash::new(&format!("{:#x}", pending.tx_hash()))
        .map_err(|e| ApiError::Internal(format!("invalid tx hash: {e}")))?;

    // Step 8: Compute contract address for deployments
    let contract_address = if request.to.is_none() {
        Some(
            ChecksumAddress::new(&format!("{:#x}", from_address.create(nonce)))
                .map_err(|e| ApiError::Internal(format!("invalid contract address: {e}")))?,
        )
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
