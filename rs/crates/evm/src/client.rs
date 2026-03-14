use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider, fillers::*};
use alloy::transports::http::reqwest::Url;

use crate::EvmError;

/// Configuration for the EVM client.
/// Services are responsible for populating this from their own config source
/// (environment variables, config files, etc.).
#[derive(Debug, Clone)]
pub struct EvmConfig {
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

/// Read-only EVM client wrapping an Alloy HTTP provider
pub struct EvmClient {
    provider: HttpProvider,
    config: EvmConfig,
}

impl EvmClient {
    pub fn new(config: EvmConfig) -> Result<Self, EvmError> {
        let url: Url = config
            .rpc_url
            .parse()
            .map_err(|e| EvmError::Config(format!("invalid rpc url: {e}")))?;

        let provider = ProviderBuilder::new().connect_http(url);

        Ok(Self { provider, config })
    }

    pub fn provider(&self) -> &HttpProvider {
        &self.provider
    }

    pub fn config(&self) -> &EvmConfig {
        &self.config
    }

    pub fn platform_contract_address(&self) -> Address {
        self.config.platform_contract_address
    }
}
