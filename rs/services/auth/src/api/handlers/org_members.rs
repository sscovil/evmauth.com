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
use crate::dto::request::{CreateOrgMember, UpdateOrgMember};
use crate::dto::response::OrgMemberResponse;
use crate::repository::filter::OrgMemberFilter;
use crate::repository::org_member::{OrgMemberRepository, OrgMemberRepositoryImpl};
use crate::AppState;

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListOrgMembersQuery {}

#[utoipa::path(
    get,
    path = "/orgs/{id}/members",
    params(
        ("id" = Uuid, Path, description = "Organization ID"),
        ListOrgMembersQuery
    ),
    responses(
        (status = 200, description = "List of organization members with pagination", body = PaginatedResponse<OrgMemberResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "org_members"
)]
pub async fn list_org_members(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListOrgMembersQuery>,
) -> Result<Json<PaginatedResponse<OrgMemberResponse>>, ApiError> {
    let repo = OrgMemberRepositoryImpl::new(&state.db);

    let page = query.to_page()?;
    let filter = OrgMemberFilter::new().org_id(org_id);
    let members = repo.list(filter, page.clone()).await?;

    let responses: Vec<OrgMemberResponse> = members.into_iter().map(|m| m.into()).collect();
    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/orgs/{id}/members",
    params(
        ("id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateOrgMember,
    responses(
        (status = 201, description = "Organization member created successfully", body = OrgMemberResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "org_members"
)]
pub async fn create_org_member(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(create): Json<CreateOrgMember>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrgMemberRepositoryImpl::new(&state.db);
    let member = repo.create(org_id, create).await?;
    Ok((StatusCode::CREATED, Json(OrgMemberResponse::from(member))))
}

#[utoipa::path(
    put,
    path = "/orgs/{org_id}/members/{member_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("member_id" = Uuid, Path, description = "Member ID")
    ),
    request_body = UpdateOrgMember,
    responses(
        (status = 200, description = "Organization member updated successfully", body = OrgMemberResponse),
        (status = 404, description = "Organization member not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "org_members"
)]
pub async fn update_org_member(
    State(state): State<AppState>,
    Path((org_id, member_id)): Path<(Uuid, Uuid)>,
    Json(update): Json<UpdateOrgMember>,
) -> Result<Json<OrgMemberResponse>, ApiError> {
    let repo = OrgMemberRepositoryImpl::new(&state.db);
    let member = repo.update(org_id, member_id, update).await?;
    Ok(Json(member.into()))
}

#[utoipa::path(
    delete,
    path = "/orgs/{org_id}/members/{member_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("member_id" = Uuid, Path, description = "Member ID")
    ),
    responses(
        (status = 204, description = "Organization member deleted successfully"),
        (status = 404, description = "Organization member not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "org_members"
)]
pub async fn delete_org_member(
    State(state): State<AppState>,
    Path((org_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let repo = OrgMemberRepositoryImpl::new(&state.db);
    repo.delete(org_id, member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
