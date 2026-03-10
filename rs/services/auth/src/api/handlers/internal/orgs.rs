use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::OrgResponse;
use crate::repository::org::{OrgRepository, OrgRepositoryImpl};

/// Get an org by ID (internal endpoint for cross-service lookups)
#[utoipa::path(
    get,
    path = "/internal/orgs/{id}",
    params(
        ("id" = Uuid, Path, description = "Org ID")
    ),
    responses(
        (status = 200, description = "Org found", body = OrgResponse),
        (status = 404, description = "Org not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/orgs"
)]
pub async fn get_org_internal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrgResponse>, ApiError> {
    let repo = OrgRepositoryImpl::new(&state.db);
    let org = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(org.into()))
}
