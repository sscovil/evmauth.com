use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider, fillers::*};
use alloy::transports::http::reqwest::Url;

use crate::ChainError;

/// Configuration for the chain client
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub platform_contract_address: Address,
    pub chain_id: u64,
}

impl ChainConfig {
    pub fn from_env() -> Result<Self, ChainError> {
        dotenvy::dotenv().ok();

        let rpc_url =
            std::env::var("RADIUS_RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());

        let contract_str = std::env::var("PLATFORM_CONTRACT_ADDRESS")
            .map_err(|_| ChainError::Config("PLATFORM_CONTRACT_ADDRESS not set".to_string()))?;

        let platform_contract_address: Address = contract_str
            .parse()
            .map_err(|e| ChainError::Config(format!("Invalid PLATFORM_CONTRACT_ADDRESS: {e}")))?;

        let chain_id: u64 = std::env::var("RADIUS_CHAIN_ID")
            .unwrap_or_else(|_| "31337".to_string())
            .parse()
            .map_err(|e| ChainError::Config(format!("Invalid RADIUS_CHAIN_ID: {e}")))?;

        Ok(Self {
            rpc_url,
            platform_contract_address,
            chain_id,
        })
    }
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
