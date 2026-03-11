pub mod client;
pub mod evmauth;

pub use alloy::primitives::{Address, Bytes, U256};
pub use client::{ChainClient, ChainConfig};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Contract error: {0}")]
    Contract(String),
}
