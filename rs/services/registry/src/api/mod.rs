pub mod error;
pub(crate) mod handlers;
pub mod openapi;
pub mod routes;

use axum::Router;

pub fn create_router(state: crate::AppState) -> Router<crate::AppState> {
    routes::api_routes(state)
}
