use axum::{
    Json, Router, middleware as axum_middleware,
    routing::{get, post, put},
};
use utoipa::OpenApi;

use crate::AppState;

use super::handlers::{
    auth as auth_handlers, end_user, health, jwks, me, org_members, orgs, people,
};

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

pub fn api_routes(state: AppState) -> Router<AppState> {
    let router = Router::new()
        .route("/openapi.json", get(openapi_spec))
        .route("/health", get(health::health_check))
        // Auth routes (no session required)
        .route("/auth/signup", post(auth_handlers::signup))
        .route("/auth/login", post(auth_handlers::login))
        .route("/auth/logout", post(auth_handlers::logout))
        // End-user auth routes (OAuth/PKCE flow)
        .route(
            "/auth/end-user/authorize",
            get(end_user::authorize).post(end_user::authenticate),
        )
        .route("/auth/end-user/token", post(end_user::token_exchange))
        // JWKS
        .route("/.well-known/jwks.json", get(jwks::jwks))
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
