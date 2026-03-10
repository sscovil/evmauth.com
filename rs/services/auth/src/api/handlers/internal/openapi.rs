use pagination::PaginatedResponse;
use utoipa::OpenApi;

use crate::dto::response::{EntityResponse, OrgResponse, PersonResponse};

/// OpenAPI documentation for Internal API endpoints
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::internal::list_entities,
        crate::api::handlers::internal::get_entity,
        crate::api::handlers::internal::delete_entity,
        crate::api::handlers::internal::people::get_person_internal,
        crate::api::handlers::internal::orgs::get_org_internal,
    ),
    components(
        schemas(
            EntityResponse,
            PaginatedResponse<EntityResponse>,
            PersonResponse,
            OrgResponse,
        )
    ),
    tags(
        (name = "internal/entities", description = "Internal entity management endpoints"),
        (name = "internal/people", description = "Internal person lookup endpoints"),
        (name = "internal/orgs", description = "Internal org lookup endpoints")
    )
)]
pub struct InternalApiDoc;
