# OpenAPI Documentation with utoipa

This document explains how to add automatic OpenAPI spec generation to Rust services in this workspace using [utoipa](https://docs.rs/utoipa/latest/utoipa/).

## Overview

Each service that has a REST API has been configured with `utoipa` to automatically generate [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.0) specifications from code annotations.

The [API gateway](./services/gateway) service provides a unified OpenAPI spec, and documents all endpoints, request/response schemas, and parameters using [Swagger UI](https://swagger.io/tools/swagger-ui/).

## Implementation Pattern

### 1. Dependencies

Dependencies are already configured at the workspace level in `rs/Cargo.toml`:

```toml
# OpenAPI documentation
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-axum = "0.1"
```

For each service, add to `Cargo.toml`:

```toml
# OpenAPI documentation
utoipa = { workspace = true }
utoipa-axum = { workspace = true }
```

### 2. Annotate DTOs with ToSchema

Add `#[derive(ToSchema)]` to all request and response DTOs and document individual fields using the `#[schema(...)]` attribute:

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePerson {
    /// The person's display name
    #[schema(
        example = "John Doe",
        format = "string"
    )]
    pub display_name: String,

    /// The person's primary email address
    #[schema(
        example = "john.doe@example.com",
        format = "email"
    )]
    pub primary_email: String,

    /// The person's role in the organization
    #[schema(
        example = "owner",
        format = "string",
        pattern = "^(owner|admin|member)$"
    )]
    pub role: String,
}
```

**Available `#[schema(...)]` attributes:**
- `example` - Example value shown in documentation
- `default` - Default value if not provided
- `minimum`, `maximum` - Numeric range constraints
- `min_length`, `max_length` - String length constraints
- `pattern` - Regex pattern for validation
- `format` - Data format hint (e.g., "email", "uuid", "date-time")
- `read_only` - Mark field as read-only (response only)
- `write_only` - Mark field as write-only (request only)

### 3. Annotate Query Parameters with IntoParams

For query parameter structs in handlers:

```rust
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListPeopleQuery {
    pub email: Option<String>,
    pub search: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}
```

### 4. Add utoipa::path Attributes to Handlers

Document each handler function with the `#[utoipa::path]` attribute:

```rust
#[utoipa::path(
    get,
    path = "/api/people",
    params(ListPeopleQuery),
    responses(
        (status = 200, description = "List of people with pagination", body = PaginatedResponse<PersonResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "people"
)]
pub async fn list_people(
    State(state): State<AppState>,
    Query(query): Query<ListPeopleQuery>,
) -> Result<Json<PaginatedResponse<PersonResponse>>, ApiError> {
    // ...
}
```

#### Path Attribute Examples

**GET with path parameter:**
```rust
#[utoipa::path(
    get,
    path = "/api/people/{id}",
    params(
        ("id" = Uuid, Path, description = "Person ID")
    ),
    responses(
        (status = 200, description = "Person found", body = PersonResponse),
        (status = 404, description = "Person not found")
    ),
    tag = "people"
)]
```

**POST with request body:**
```rust
#[utoipa::path(
    post,
    path = "/api/people",
    request_body = CreatePerson,
    responses(
        (status = 201, description = "Person created successfully", body = PersonResponse),
        (status = 400, description = "Bad request")
    ),
    tag = "people"
)]
```

**PUT with both path parameter and request body:**
```rust
#[utoipa::path(
    put,
    path = "/api/people/{id}",
    params(
        ("id" = Uuid, Path, description = "Person ID")
    ),
    request_body = UpdatePerson,
    responses(
        (status = 200, description = "Person updated successfully", body = PersonResponse),
        (status = 404, description = "Person not found")
    ),
    tag = "people"
)]
```

**DELETE:**
```rust
#[utoipa::path(
    delete,
    path = "/api/people/{id}",
    params(
        ("id" = Uuid, Path, description = "Person ID")
    ),
    responses(
        (status = 204, description = "Person deleted successfully"),
        (status = 404, description = "Person not found")
    ),
    tag = "people"
)]
```

### 5. Create OpenAPI Documentation Struct

Create `src/api/openapi.rs`:

```rust
use utoipa::OpenApi;

use crate::dto::request::{/* import all request DTOs */};
use crate::dto::response::{/* import all response DTOs */};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Your Service API",
        version = "1.0.0",
        description = "Your service description"
    ),
    paths(
        // List all handler functions here
        crate::api::handlers::your_module::your_handler,
        // ...
    ),
    components(
        schemas(
            // List all DTOs here
            YourRequestDTO,
            YourResponseDTO,
            PaginatedResponse<YourResponseDTO>,
            // ...
        )
    ),
    tags(
        (name = "your_tag", description = "Tag description"),
        // ...
    )
)]
pub struct ApiDoc;
```

### 6. Make Handlers Accessible

In `src/api/mod.rs`, make handlers accessible to the openapi module:

```rust
pub mod error;
pub(crate) mod handlers;  // Note: pub(crate) instead of mod
pub mod openapi;
pub mod routes;
```

### 7. Add OpenAPI Endpoint to Routes

In `src/api/routes.rs`:

```rust
use axum::{routing::get, Json, Router};
use utoipa::OpenApi;

use super::openapi::ApiDoc;

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn api_routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_spec))
        // ... other routes
}
```

## Testing the OpenAPI Spec

Once your service is running, the OpenAPI spec is available at:

```bash
# Get the OpenAPI spec
curl http://localhost:8000/api/openapi.json

# Or use jq for formatted output
curl http://localhost:8000/api/openapi.json | jq
```

## Using with Swagger UI or Other Tools

The generated OpenAPI spec is fully compliant with OpenAPI 3.1 and can be used with:

- **Swagger UI**: Import the spec URL to get an interactive API documentation
- **Postman**: Import the spec to auto-generate API collections
- **OpenAPI Generator**: Generate client libraries in various languages
- **Redoc**: Generate beautiful static documentation

## Key Features

- **Type Safety**: Changes to DTOs automatically update the OpenAPI spec
- **Zero Maintenance**: No need to manually maintain YAML/JSON spec files
- **Validation**: Compile-time checking ensures documented types match actual code
- **Framework Integration**: Works seamlessly with Axum extractors
- **Rich Annotations**: Support for descriptions, examples, deprecation, and more

## Example: Complete Handler

Here's a complete example showing all patterns:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::dto::request::{CreateResource, UpdateResource, PaginationParams};
use crate::dto::response::{ResourceResponse, PaginatedResponse};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListResourcesQuery {
    pub search: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[utoipa::path(
    get,
    path = "/api/resources",
    params(ListResourcesQuery),
    responses(
        (status = 200, description = "List of resources", body = PaginatedResponse<ResourceResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "resources"
)]
pub async fn list_resources(
    State(state): State<AppState>,
    Query(query): Query<ListResourcesQuery>,
) -> Result<Json<PaginatedResponse<ResourceResponse>>, ApiError> {
    // Implementation
}
```

## Replicating for Other Services

To add OpenAPI documentation to a new service in the workspace:

1. Add utoipa dependencies to the service's `Cargo.toml`
2. Add `ToSchema` derives to all DTOs
3. Add `IntoParams` derives to query parameter structs
4. Add `#[utoipa::path(...)]` attributes to all handler functions
5. Create `src/api/openapi.rs` with the `ApiDoc` struct
6. Update `src/api/mod.rs` to make handlers accessible
7. Add the `/openapi.json` route in `src/api/routes.rs`
8. Build and test

The pattern is consistent across all services, making it easy to maintain documentation as your API evolves.

## Reference

- [utoipa Documentation](https://docs.rs/utoipa/latest/utoipa/)
- [utoipa-axum Documentation](https://docs.rs/utoipa-axum/latest/utoipa_axum/)
- [OpenAPI 3.1 Specification](https://spec.openapis.org/oas/v3.1.0)
