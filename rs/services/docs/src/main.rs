use docs::{aggregator::Aggregator, config::Config, routes};
use std::sync::Arc;

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

    // Create aggregator
    let aggregator = Aggregator::new();

    // Create application state
    let state = Arc::new(routes::AppState { config, aggregator });

    // Create the router
    let router = routes::create_router(state.clone());

    // Start the server
    let addr = format!("0.0.0.0:{}", state.config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("API Documentation server listening on {}", addr);
    tracing::info!(
        "Visit http://localhost:{} to view the documentation",
        state.config.port
    );

    axum::serve(listener, router).await?;

    Ok(())
}
