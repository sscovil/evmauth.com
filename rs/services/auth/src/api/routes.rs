use axum::{
    Json, Router, middleware as axum_middleware,
    routing::{get, post, put},
};
use utoipa::OpenApi;

use crate::AppState;

use super::handlers::{
    auth as auth_handlers, challenges, end_users, health, me, org_members, orgs, people,
};

#[cfg(feature = "internal-api")]
use super::handlers::internal;
use super::openapi::ApiDoc;

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    let spec = ApiDoc::openapi();
    Json(merge_internal_spec(spec))
}

#[cfg(feature = "internal-api")]
fn merge_internal_spec(mut spec: utoipa::openapi::OpenApi) -> utoipa::openapi::OpenApi {
    use super::handlers::internal::InternalApiDoc;
    spec.merge(InternalApiDoc::openapi());
    spec
}

#[cfg(not(feature = "internal-api"))]
fn merge_internal_spec(spec: utoipa::openapi::OpenApi) -> utoipa::openapi::OpenApi {
    spec
}

pub fn api_routes(state: AppState) -> Router<AppState> {
    let router = Router::new()
        .route("/openapi.json", get(openapi_spec))
        .route("/health", get(health::health_check))
        // Auth routes (no session required)
        .route("/challenges", post(challenges::create_challenge))
        .route(
            "/sessions",
            post(auth_handlers::login).delete(auth_handlers::logout),
        )
        // End-user onboarding
        .route("/end-users", post(end_users::create_end_user))
        // People routes
        .route(
            "/people",
            get(people::list_people).post(auth_handlers::signup),
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
        )
        // Protected "me" routes
        .route(
            "/me",
            get(me::get_me)
                .patch(me::update_me)
                .route_layer(axum_middleware::from_fn_with_state(
                    state.clone(),
                    crate::middleware::require_session,
                )),
        )
        .route(
            "/me/authenticators",
            post(me::create_authenticator).route_layer(axum_middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::require_session,
            )),
        );

    // Conditionally add internal routes with /internal prefix
    #[cfg(feature = "internal-api")]
    let router = router
        .route("/internal/entities", get(internal::list_entities))
        .route(
            "/internal/entities/{id}",
            get(internal::get_entity).delete(internal::delete_entity),
        )
        .route("/internal/people/{id}", get(internal::get_person_internal))
        .route("/internal/orgs/{id}", get(internal::get_org_internal));

    router
}
