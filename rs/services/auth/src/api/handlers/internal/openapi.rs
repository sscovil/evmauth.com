use pagination::PaginatedResponse;
use utoipa::OpenApi;

use crate::dto::response::EntityResponse;

/// OpenAPI documentation for Internal API endpoints
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::internal::list_entities,
        crate::api::handlers::internal::get_entity,
        crate::api::handlers::internal::delete_entity,
    ),
    components(
        schemas(
            EntityResponse,
            PaginatedResponse<EntityResponse>,
        )
    ),
    tags(
        (name = "internal/entities", description = "Internal entity management endpoints")
    )
)]
pub struct InternalApiDoc;
