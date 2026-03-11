use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider, fillers::*};
use alloy::transports::http::reqwest::Url;

use crate::ChainError;

/// Configuration for the chain client.
/// Services are responsible for populating this from their own config source
/// (environment variables, config files, etc.).
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub platform_contract_address: Address,
    pub chain_id: u64,
}

type HttpProvider = FillProvider<
    JoinFill<
        alloy::providers::Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
>;

/// Read-only chain client wrapping an Alloy HTTP provider
pub struct ChainClient {
    provider: HttpProvider,
    config: ChainConfig,
}

impl ChainClient {
    pub fn new(config: ChainConfig) -> Result<Self, ChainError> {
        let url: Url = config
            .rpc_url
            .parse()
            .map_err(|e| ChainError::Config(format!("Invalid RPC URL: {e}")))?;

        let provider = ProviderBuilder::new().connect_http(url);

        Ok(Self { provider, config })
    }

    pub fn provider(&self) -> &HttpProvider {
        &self.provider
    }

    pub fn config(&self) -> &ChainConfig {
        &self.config
    }

    pub fn platform_contract_address(&self) -> Address {
        self.config.platform_contract_address
    }
}
