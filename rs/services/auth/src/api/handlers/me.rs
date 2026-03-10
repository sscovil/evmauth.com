use axum::{Extension, Json, extract::State};

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::PersonResponse;
use crate::middleware::AuthenticatedPerson;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};

/// Get current person profile
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "Current person profile", body = PersonResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Person not found")
    ),
    tag = "me",
    security(("session" = []))
)]
pub async fn get_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedPerson>,
) -> Result<Json<PersonResponse>, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo.get(auth.person_id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(person.into()))
}

/// Update display name request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateMeRequest {
    /// New display name
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: Option<String>,
    /// New description
    #[schema(
        example = "Software engineer and open source contributor",
        format = "string"
    )]
    pub description: Option<String>,
}

/// Update current person profile
#[utoipa::path(
    patch,
    path = "/me",
    request_body = UpdateMeRequest,
    responses(
        (status = 200, description = "Profile updated", body = PersonResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Person not found")
    ),
    tag = "me",
    security(("session" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedPerson>,
    Json(body): Json<UpdateMeRequest>,
) -> Result<Json<PersonResponse>, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo
        .update(
            auth.person_id,
            crate::dto::request::UpdatePerson {
                display_name: body.display_name,
                description: body.description,
                primary_email: None,
            },
        )
        .await?;
    Ok(Json(person.into()))
}
