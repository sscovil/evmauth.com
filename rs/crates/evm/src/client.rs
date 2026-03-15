use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider, fillers::*};
use alloy::transports::http::reqwest::Url;

use crate::EvmError;

/// Configuration for the EVM client.
/// Services are responsible for populating this from their own config source
/// (environment variables, config files, etc.).
#[derive(Debug, Clone)]
pub struct EvmConfig {
    /// JSON-RPC endpoint URL for the target EVM chain (e.g. `https://rpc.example.com`).
    pub rpc_url: String,
    /// On-chain address of the deployed EVMAuth6909 platform contract (beacon).
    pub platform_contract_address: Address,
    /// On-chain address of the platform operator wallet used for routine operations
    /// (deploying proxies, minting/burning tokens). Receives non-admin roles on new proxies.
    pub platform_operator_address: Address,
    /// Numeric chain ID used for EIP-155 transaction signing.
    pub chain_id: u64,
}

type HttpProvider = FillProvider<
    JoinFill<
        alloy::providers::Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
>;

/// Read-only EVM client wrapping an Alloy HTTP provider
pub struct EvmClient {
    provider: HttpProvider,
    config: EvmConfig,
}

impl EvmClient {
    /// Build a new client by parsing the RPC URL from `config` and connecting an HTTP provider.
    pub fn new(config: EvmConfig) -> Result<Self, EvmError> {
        let url: Url = config
            .rpc_url
            .parse()
            .map_err(|e| EvmError::Config(format!("invalid rpc url: {e}")))?;

        let provider = ProviderBuilder::new().connect_http(url);

        Ok(Self { provider, config })
    }

    /// Returns a reference to the underlying Alloy HTTP provider for direct RPC calls.
    pub fn provider(&self) -> &HttpProvider {
        &self.provider
    }

    /// Returns a reference to the configuration this client was built with.
    pub fn config(&self) -> &EvmConfig {
        &self.config
    }

    /// Shorthand for `self.config().platform_contract_address`.
    pub fn platform_contract_address(&self) -> Address {
        self.config.platform_contract_address
    }

    /// Shorthand for `self.config().platform_operator_address`.
    pub fn platform_operator_address(&self) -> Address {
        self.config.platform_operator_address
    }
}
