use std::sync::Arc;

use registry::{AppState, api, config};

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

    let redis = redis_client::create_client(&config.redis.connection_string()).await?;
    tracing::info!("Redis connection established");

    let chain = chain::ChainClient::new(config.chain.clone())?;
    tracing::info!("Chain client initialized");

    let http_client = reqwest::Client::new();

    let state = AppState {
        db,
        redis,
        chain: Arc::new(chain),
        http_client,
        config: Arc::new(config),
    };

    let router = api::create_router(state.clone()).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}
