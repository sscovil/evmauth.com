use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response from the wallets service send-tx endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendTxResponse {
    /// The transaction hash
    #[schema(example = "0xabc123...")]
    pub tx_hash: String,

    /// The deployed contract address (populated for contract creation)
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub contract_address: Option<String>,
}
