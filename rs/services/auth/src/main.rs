use auth::{AppState, api, config, jwt};
use std::sync::Arc;

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

    // Load JWT keys (optional in dev)
    let jwt_keys = match (&config.jwt_private_key_pem, &config.jwt_public_key_pem) {
        (Some(private), Some(public)) => {
            let keys = jwt::JwtKeys::from_pem(private.as_bytes(), public.as_bytes())?;
            tracing::info!("JWT keys loaded");
            Some(Arc::new(keys))
        }
        _ => {
            tracing::warn!("JWT keys not configured - auth endpoints will be unavailable");
            None
        }
    };

    // Create chain client
    let chain_client = chain::ChainClient::new(config.chain.clone())
        .map_err(|e| anyhow::anyhow!("Failed to create chain client: {e}"))?;
    tracing::info!(
        contract = %config.chain.platform_contract_address,
        chain_id = config.chain.chain_id,
        rpc_url = %config.chain.rpc_url,
        "Chain client initialized"
    );

    // Create HTTP client for internal service calls
    let http_client = reqwest::Client::new();

    // Create application state
    let state = AppState {
        db,
        redis,
        jwt_keys,
        http_client,
        config: Arc::new(config),
        chain: Arc::new(chain_client),
    };

    // Create the router
    let router = api::create_router(state.clone()).with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}
