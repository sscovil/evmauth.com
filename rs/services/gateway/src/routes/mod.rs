use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::json;
use std::sync::Arc;

use crate::proxy::handler::{AppState, proxy_handler};

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Use service-discovery crate for health checking
    let health = service_discovery::check_all_services_health(
        &state.client,
        &state.config.services,
        &["/health"],
        2, // 2 second timeout
    )
    .await;

    let status_code = if health.status.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": match health.status {
                service_discovery::OverallStatus::Healthy => "healthy",
                service_discovery::OverallStatus::Degraded => "degraded",
            },
            "services": health.services,
        })),
    )
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .fallback(proxy_handler) // Catch-all for proxy
        .with_state(state)
}
