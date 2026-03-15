use anyhow::Result;

/// Sends ETH from Anvil account #0 to the given address.
/// This uses the well-known Anvil default private key and is for local development only.
pub async fn execute(address: &str, amount: &str) -> Result<()> {
    println!("Funding {address} with {amount} ETH from Anvil account #0...");
    // TODO: Implement using alloy provider with Anvil default key
    anyhow::bail!("Not yet implemented -- will be built in Phase 2")
}
