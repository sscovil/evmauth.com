pub mod error;
pub mod handlers;
pub mod openapi;
pub mod routes;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::AppState;

pub fn create_router(state: AppState) -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    routes::api_routes(state).layer(cors)
}
