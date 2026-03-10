use assets::{AppState, api, config, s3::S3Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

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

    let s3 = S3Client::new(&config.s3).await?;
    tracing::info!("S3 client initialized");

    let state = AppState { db, redis, s3 };

    let router = api::create_router(state.clone()).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}
