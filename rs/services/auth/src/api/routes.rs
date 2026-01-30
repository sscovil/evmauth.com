use axum::{
    routing::{get, put},
    Json, Router,
};
use utoipa::OpenApi;

use crate::AppState;

use super::handlers::{health, org_members, orgs, people};

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
        // People routes
        .route(
            "/people",
            get(people::list_people).post(people::create_person),
        )
        .route(
            "/people/{id}",
            get(people::get_person)
                .put(people::update_person)
                .delete(people::delete_person),
        )
        // Orgs routes
        .route("/orgs", get(orgs::list_orgs).post(orgs::create_org))
        .route(
            "/orgs/{id}",
            get(orgs::get_org)
                .put(orgs::update_org)
                .delete(orgs::delete_org),
        )
        // Org members routes
        .route(
            "/orgs/{id}/members",
            get(org_members::list_org_members).post(org_members::create_org_member),
        )
        .route(
            "/orgs/{org_id}/members/{member_id}",
            put(org_members::update_org_member).delete(org_members::delete_org_member),
        );

    // Conditionally add internal routes with /internal prefix
    #[cfg(feature = "internal-api")]
    let router = router
        .route("/internal/entities", get(internal::list_entities))
        .route(
            "/internal/entities/{id}",
            get(internal::get_entity).delete(internal::delete_entity),
        );

    router
}
