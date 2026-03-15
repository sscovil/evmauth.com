use alloy::hex;
use alloy::primitives::{Address, Bytes};
use alloy::sol;
use alloy::sol_types::SolConstructor;

use crate::EvmError;

/// OpenZeppelin BeaconProxy creation bytecode (hex-encoded with 0x prefix).
/// Source: @openzeppelin/contracts/proxy/beacon/BeaconProxy.sol
const BEACON_PROXY_BYTECODE_HEX: &str = include_str!("../artifacts/BeaconProxy.bin");

sol! {
    /// Minimal BeaconProxy constructor ABI for encoding constructor arguments.
    contract BeaconProxy {
        constructor(address beacon, bytes memory data);
    }
}

/// Returns the full deployment bytecode for a BeaconProxy contract.
///
/// The `init_data` parameter is typically empty (`Bytes::new()`) when no
/// initializer call is needed (e.g., for EVMAuth6909 proxies).
pub fn encode_beacon_proxy_deploy(beacon: Address, init_data: Bytes) -> Result<Bytes, EvmError> {
    let hex_str = BEACON_PROXY_BYTECODE_HEX
        .trim()
        .strip_prefix("0x")
        .unwrap_or(BEACON_PROXY_BYTECODE_HEX.trim());
    let creation_code = hex::decode(hex_str).map_err(|e| {
        EvmError::Config(format!(
            "invalid beacon proxy bytecode hex in artifact: {e}"
        ))
    })?;

    let args = BeaconProxy::constructorCall {
        beacon,
        data: init_data,
    };
    let encoded_args = args.abi_encode();

    let mut deploy_bytecode = creation_code;
    deploy_bytecode.extend_from_slice(&encoded_args);
    Ok(Bytes::from(deploy_bytecode))
}
