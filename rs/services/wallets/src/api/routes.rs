#[cfg(feature = "internal-api")]
use axum::routing::post;
use axum::{Json, Router, routing::get};
use utoipa::OpenApi;

use crate::AppState;

use super::handlers::{health, org_wallets, person_wallets};

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
        // Org wallet routes
        .route("/orgs/{org_id}/wallet", get(org_wallets::get_org_wallet))
        // Person wallet routes (placeholder, needs auth middleware later)
        .route("/me/wallets", get(person_wallets::list_my_wallets))
        .route("/me/wallets/{app_id}", get(person_wallets::get_my_wallet));

    // Conditionally add internal routes with /internal prefix
    #[cfg(feature = "internal-api")]
    let router = router
        .route(
            "/internal/person-sub-org",
            post(internal::person_sub_orgs::create_person_sub_org),
        )
        .route(
            "/internal/org-wallet",
            post(internal::org_wallets::create_org_wallet),
        )
        .route(
            "/internal/org-wallet/{org_id}",
            get(internal::org_wallets::get_org_wallet_internal),
        )
        .route(
            "/internal/person-app-wallet",
            post(internal::person_app_wallets::create_person_app_wallet),
        )
        .route(
            "/internal/person-app-wallet/{id}",
            get(internal::person_app_wallets::get_person_app_wallet),
        )
        .route("/internal/sign", post(internal::signing::sign_payload));

    router
}
