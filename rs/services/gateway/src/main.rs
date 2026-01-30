use gateway::{config::Config, proxy::handler::AppState, routes};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");
    tracing::info!("Discovered {} service(s)", config.services.len());
    for service in &config.services {
        tracing::info!("  - {}: {}", service.name, service.base_url);
    }

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .build()?;

    // Create application state
    let state = Arc::new(AppState { config, client });

    // Create the router
    let router = routes::create_router(state.clone());

    // Start the server
    let addr = format!("0.0.0.0:{}", state.config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("API Gateway listening on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}
