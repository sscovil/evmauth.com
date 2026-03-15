use serde::{Deserialize, Serialize};
use types::{ChecksumAddress, TxHash};
use utoipa::ToSchema;

/// Response from the wallets service send-tx endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendTxResponse {
    /// The transaction hash
    #[schema(example = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab")]
    pub tx_hash: TxHash,

    /// The deployed contract address (populated for contract creation)
    #[schema(example = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed")]
    pub contract_address: Option<ChecksumAddress>,
}
