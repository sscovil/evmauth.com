use std::sync::Arc;

use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wallets::{AppState, api, config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded");

    let db = postgres::create_pool(
        &config.pg.connection_string(),
        config.pg.pool.max_connections,
        config.pg.pool.min_connections,
    )
    .await?;
    tracing::info!("PostgreSQL connection established");

    // Connect to Redis
    let redis = redis_client::create_client(&config.redis.connection_string()).await?;
    tracing::info!("Redis connection established");

    // Create Turnkey client using official SDK
    let api_key = TurnkeyP256ApiKey::from_strings(
        &config.turnkey.api_private_key,
        Some(&config.turnkey.api_public_key),
    )?;
    let turnkey = turnkey_client::TurnkeyClient::builder()
        .api_key(api_key)
        .base_url(&config.turnkey.api_base_url)
        .build()?;
    tracing::info!("Turnkey client initialized");

    // Create EVM client
    let evm_client = evm::EvmClient::new(config.evm)?;
    tracing::info!("EVM client initialized");

    // Create application state
    let state = AppState {
        db,
        redis,
        turnkey: Arc::new(turnkey),
        turnkey_parent_org_id: config.turnkey.parent_org_id,
        evm: Arc::new(evm_client),
    };

    // Create the router
    let router = api::create_router(state.clone()).with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}
