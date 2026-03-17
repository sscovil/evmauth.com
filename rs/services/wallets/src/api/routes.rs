#[cfg(feature = "internal-api")]
use axum::routing::post;
use axum::{Json, Router, routing::get};
use utoipa::OpenApi;

use crate::AppState;

use super::handlers::{entity_app_wallets, entity_wallets, health};

#[cfg(feature = "internal-api")]
use super::handlers::internal;
use super::openapi::ApiDoc;

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    #[cfg(feature = "internal-api")]
    {
        use super::handlers::internal::InternalApiDoc;
        let mut spec = ApiDoc::openapi();
        spec.merge(InternalApiDoc::openapi());
        return Json(spec);
    }

    #[cfg(not(feature = "internal-api"))]
    Json(ApiDoc::openapi())
}

pub fn api_routes(_state: AppState) -> Router<AppState> {
    let router = Router::new()
        .route("/openapi.json", get(openapi_spec))
        .route("/health", get(health::health_check))
        // Org wallet routes (org_id is the entity_id for orgs)
        .route("/orgs/{org_id}/wallet", get(entity_wallets::get_org_wallet))
        // Person wallet routes (placeholder, needs auth middleware later)
        .route("/me/wallets", get(entity_app_wallets::list_my_wallets))
        .route(
            "/me/wallets/{app_id}",
            get(entity_app_wallets::get_my_wallet),
        );

    // Conditionally add internal routes with /internal prefix
    #[cfg(feature = "internal-api")]
    let router = router
        .route(
            "/internal/entity-wallet",
            post(internal::entity_wallets::create_entity_wallet),
        )
        .route(
            "/internal/entity-wallet/{entity_id}",
            get(internal::entity_wallets::get_entity_wallet),
        )
        .route(
            "/internal/entity-app-wallet",
            post(internal::entity_app_wallets::create_entity_app_wallet),
        )
        .route(
            "/internal/entity-app-wallet/{entity_id}/{app_id}",
            get(internal::entity_app_wallets::get_entity_app_wallet),
        )
        .route(
            "/internal/signatures",
            post(internal::signing::sign_payload),
        )
        .route("/internal/transactions", post(internal::send_tx::send_tx))
        .route(
            "/internal/authenticators",
            post(internal::authenticators::create_authenticators),
        );

    router
}
