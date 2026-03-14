pub mod beacon;
pub mod client;
pub mod evmauth;

pub use alloy::primitives::{Address, Bytes, U256};
pub use beacon::encode_beacon_proxy_deploy;
pub use client::{EvmClient, EvmConfig};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvmError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("transport error: {0}")]
    Transport(String),

    #[error("contract error: {0}")]
    Contract(String),

    #[error("rpc timeout: {0}")]
    Timeout(String),
}
