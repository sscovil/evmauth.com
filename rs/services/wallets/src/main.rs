use std::sync::Arc;

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

    // Create Turnkey client
    let turnkey = turnkey::TurnkeyClient::new(config.turnkey)?;
    tracing::info!("Turnkey client initialized");

    // Create application state
    let state = AppState {
        db,
        redis,
        turnkey: Arc::new(turnkey),
    };

    // Create the router
    let router = api::create_router(state.clone()).with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}
