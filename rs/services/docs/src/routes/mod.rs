use crate::{aggregator::Aggregator, config::Config};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use std::sync::Arc;

pub struct AppState {
    pub config: Config,
    pub aggregator: Aggregator,
}

/// Serve Swagger UI HTML
async fn swagger_ui() -> Html<&'static str> {
    Html(include_str!("../static/swagger.html"))
}

/// Serve merged OpenAPI spec
async fn merged_spec(State(state): State<Arc<AppState>>) -> Response {
    let specs = state
        .aggregator
        .fetch_all_specs(&state.config.services)
        .await;
    let merged = state
        .aggregator
        .merge_specs(specs, &state.config.api_config);

    Json(merged).into_response()
}

/// List available services
async fn list_services(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<crate::config::ServiceConfig>> {
    Json(state.config.services.clone())
}

/// Health check endpoint that also checks all discovered services
async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = reqwest::Client::new();

    // Use service-discovery crate for health checking
    let health = service_discovery::check_all_services_health(
        &client,
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
        Json(serde_json::json!({
            "status": match health.status {
                service_discovery::OverallStatus::Healthy => "healthy",
                service_discovery::OverallStatus::Degraded => "degraded",
            },
            "services": health.services,
        })),
    )
}

/// Create the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(swagger_ui))
        .route("/openapi.json", get(merged_spec))
        .route("/services", get(list_services))
        .route("/health", get(health))
        .with_state(state)
}
