use alloy::primitives::{Address, FixedBytes, address, eip191_hash_message};
use alloy::signers::Signature;
use alloy::sol;
use alloy::sol_types::SolStruct;

use crate::EvmError;

sol! {
    /// EIP-712 typed data for EVMAuth API account queries.
    #[sol(all_derives)]
    struct AccountsQuery {
        address account;
        address contract;
        string client_id;
        bytes32 nonce;
    }
}

/// EIP-712 domain name used for EVMAuth API requests.
const EIP712_DOMAIN_NAME: &str = "EVMAuth";
/// EIP-712 domain version.
const EIP712_DOMAIN_VERSION: &str = "1";
/// EIP-712 verifying contract (zero address -- no on-chain verifier).
const EIP712_VERIFYING_CONTRACT: Address = address!("0x0000000000000000000000000000000000000000");

/// Recover the signer address from an EIP-191 personal_sign message and signature.
///
/// The `message` is the raw message bytes (the "\x19Ethereum Signed Message:\n" prefix
/// is applied internally). The `signature_bytes` is the 65-byte compact signature (r ++ s ++ v).
pub fn recover_signer(message: &[u8], signature_bytes: &[u8]) -> Result<Address, EvmError> {
    let sig = Signature::from_raw(signature_bytes)
        .map_err(|e| EvmError::Contract(format!("invalid signature: {e}")))?;
    let hash = eip191_hash_message(message);
    sig.recover_address_from_prehash(&hash)
        .map_err(|e| EvmError::Contract(format!("ecrecover failed: {e}")))
}

/// Build the EIP-712 domain separator for EVMAuth API requests on a given chain.
fn eip712_domain(chain_id: u64) -> alloy::sol_types::Eip712Domain {
    alloy::sol_types::Eip712Domain {
        name: Some(EIP712_DOMAIN_NAME.into()),
        version: Some(EIP712_DOMAIN_VERSION.into()),
        chain_id: Some(alloy::primitives::U256::from(chain_id)),
        verifying_contract: Some(EIP712_VERIFYING_CONTRACT),
        salt: None,
    }
}

/// Verify an ERC-712 typed-data signature for an accounts query.
///
/// Returns the recovered signer address on success.
pub fn verify_accounts_query(
    account: Address,
    contract: Address,
    client_id: &str,
    nonce: FixedBytes<32>,
    chain_id: u64,
    signature_bytes: &[u8],
) -> Result<Address, EvmError> {
    let domain = eip712_domain(chain_id);
    let data = AccountsQuery {
        account,
        contract,
        client_id: client_id.to_string(),
        nonce,
    };

    let signing_hash = data.eip712_signing_hash(&domain);
    let sig = Signature::from_raw(signature_bytes)
        .map_err(|e| EvmError::Contract(format!("invalid signature: {e}")))?;

    sig.recover_address_from_prehash(&signing_hash)
        .map_err(|e| EvmError::Contract(format!("ERC-712 ecrecover failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recover_signer_rejects_short_signature() {
        let result = recover_signer(b"hello", &[0u8; 64]);
        assert!(result.is_err());
    }
}
