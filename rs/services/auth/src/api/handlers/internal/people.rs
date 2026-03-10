use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::PersonResponse;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};

/// Get a person by ID (internal endpoint for cross-service lookups)
#[utoipa::path(
    get,
    path = "/internal/people/{id}",
    params(
        ("id" = Uuid, Path, description = "Person ID")
    ),
    responses(
        (status = 200, description = "Person found", body = PersonResponse),
        (status = 404, description = "Person not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/people"
)]
pub async fn get_person_internal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PersonResponse>, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(person.into()))
}
