/// Beacon proxy deployment bytecode encoding.
pub mod beacon;
/// EVM client configuration and HTTP provider wrapper.
pub mod client;
/// EVMAuth6909 contract interaction helpers (queries and calldata encoding).
pub mod evmauth;

/// Re-export of Alloy address type for use by dependent crates.
pub use alloy::primitives::{Address, Bytes, U256};
/// Re-export beacon proxy deploy encoder for convenience.
pub use beacon::encode_beacon_proxy_deploy;
/// Re-export the EVM client and its configuration struct.
pub use client::{EvmClient, EvmConfig};
/// Re-export EVMAuth role constants and helpers.
pub use evmauth::roles;

use thiserror::Error;

/// Errors that can occur when interacting with EVM contracts or configuring the client.
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
