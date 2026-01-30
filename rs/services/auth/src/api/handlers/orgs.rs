use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use pagination::{with_pagination, PaginatedResponse};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::domain::OrgVisibility;
use crate::dto::request::{CreateOrg, UpdateOrg};
use crate::dto::response::OrgResponse;
use crate::repository::filter::OrgFilter;
use crate::repository::org::{OrgRepository, OrgRepositoryImpl};
use crate::AppState;

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListOrgsQuery {
    pub owner_id: Option<Uuid>,
    pub visibility: Option<OrgVisibility>,
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/orgs",
    params(ListOrgsQuery),
    responses(
        (status = 200, description = "List of organizations with pagination", body = PaginatedResponse<OrgResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "orgs"
)]
pub async fn list_orgs(
    State(state): State<AppState>,
    Query(query): Query<ListOrgsQuery>,
) -> Result<Json<PaginatedResponse<OrgResponse>>, ApiError> {
    let repo = OrgRepositoryImpl::new(&state.db);

    let page = query.to_page()?;

    let mut filter = OrgFilter::new();
    if let Some(ref owner_id) = query.owner_id {
        filter = filter.owner_id(*owner_id);
    }
    if let Some(visibility) = query.visibility {
        filter = filter.visibility(visibility);
    }
    if let Some(ref search) = query.search {
        filter = filter.search(search.clone());
    }

    let orgs = repo.list(filter, page.clone()).await?;

    let responses: Vec<OrgResponse> = orgs.into_iter().map(|o| o.into()).collect();
    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/orgs/{id}",
    params(
        ("id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Organization found", body = OrgResponse),
        (status = 404, description = "Organization not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "orgs"
)]
pub async fn get_org(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrgResponse>, ApiError> {
    let repo = OrgRepositoryImpl::new(&state.db);
    let org = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(org.into()))
}

#[utoipa::path(
    post,
    path = "/orgs",
    request_body = CreateOrg,
    responses(
        (status = 201, description = "Organization created successfully", body = OrgResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "orgs"
)]
pub async fn create_org(
    State(state): State<AppState>,
    Json(create): Json<CreateOrg>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrgRepositoryImpl::new(&state.db);
    let org = repo.create(create).await?;
    Ok((StatusCode::CREATED, Json(OrgResponse::from(org))))
}

#[utoipa::path(
    put,
    path = "/orgs/{id}",
    params(
        ("id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = UpdateOrg,
    responses(
        (status = 200, description = "Organization updated successfully", body = OrgResponse),
        (status = 404, description = "Organization not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "orgs"
)]
pub async fn update_org(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateOrg>,
) -> Result<Json<OrgResponse>, ApiError> {
    let repo = OrgRepositoryImpl::new(&state.db);
    let org = repo.update(id, update).await?;
    Ok(Json(org.into()))
}

#[utoipa::path(
    delete,
    path = "/orgs/{id}",
    params(
        ("id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 404, description = "Organization not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "orgs"
)]
pub async fn delete_org(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = OrgRepositoryImpl::new(&state.db);
    repo.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
