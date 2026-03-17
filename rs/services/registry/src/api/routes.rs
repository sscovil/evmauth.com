#[cfg(feature = "internal-api")]
use axum::routing::get as get_route;
use axum::{
    Json, Router,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::AppState;

use super::handlers::{app_registrations, contracts, health};

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
        // App registration routes
        .route(
            "/orgs/{org_id}/apps",
            post(app_registrations::create_app_registration)
                .get(app_registrations::list_app_registrations),
        )
        .route(
            "/orgs/{org_id}/apps/{app_id}",
            get(app_registrations::get_app_registration)
                .patch(app_registrations::update_app_registration)
                .delete(app_registrations::delete_app_registration),
        )
        // Contract routes
        .route(
            "/orgs/{org_id}/contracts",
            post(contracts::deploy_contract).get(contracts::list_contracts),
        )
        .route(
            "/orgs/{org_id}/contracts/{contract_id}",
            get(contracts::get_contract),
        )
        // Role grant routes
        .route(
            "/orgs/{org_id}/contracts/{contract_id}/roles",
            post(contracts::create_role_grant).get(contracts::list_role_grants),
        )
        .route(
            "/orgs/{org_id}/contracts/{contract_id}/roles/{role_grant_id}",
            axum::routing::delete(contracts::delete_role_grant),
        );

    #[cfg(feature = "internal-api")]
    let router = router
        .route(
            "/internal/apps/by-client-id/{client_id}",
            get_route(internal::app_registrations::get_app_by_client_id),
        )
        .route(
            "/internal/contracts/{id}",
            get_route(internal::contracts::get_contract_internal),
        )
        .route(
            "/internal/contracts/by-app/{app_registration_id}",
            get_route(internal::contracts::get_contract_by_app_registration_id),
        );

    router
}
