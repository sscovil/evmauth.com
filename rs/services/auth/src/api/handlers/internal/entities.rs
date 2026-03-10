use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use pagination::{PaginatedResponse, with_pagination};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::EntityResponse;
use crate::repository::entity::{EntityRepository, EntityRepositoryImpl};
use crate::repository::filter::EntityFilter;

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListEntitiesQuery {
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/internal/entities",
    params(ListEntitiesQuery),
    responses(
        (status = 200, description = "List of entities with pagination", body = PaginatedResponse<EntityResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entities"
)]
pub async fn list_entities(
    State(state): State<AppState>,
    Query(query): Query<ListEntitiesQuery>,
) -> Result<Json<PaginatedResponse<EntityResponse>>, ApiError> {
    let repo = EntityRepositoryImpl::new(&state.db);

    let page = query.to_page()?;

    let mut filter = EntityFilter::new();
    if let Some(ref entity_type) = query.entity_type {
        filter = filter.entity_type(entity_type.clone());
    }
    if let Some(ref search) = query.search {
        filter = filter.search(search.clone());
    }

    let entities = repo.list(filter, page.clone()).await?;

    let responses: Vec<EntityResponse> = entities.into_iter().map(|e| e.into()).collect();
    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/internal/entities/{id}",
    params(
        ("id" = Uuid, Path, description = "Entity ID")
    ),
    responses(
        (status = 200, description = "Entity found", body = EntityResponse),
        (status = 404, description = "Entity not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entities"
)]
pub async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EntityResponse>, ApiError> {
    let repo = EntityRepositoryImpl::new(&state.db);
    let entity = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(entity.into()))
}

#[utoipa::path(
    delete,
    path = "/internal/entities/{id}",
    params(
        ("id" = Uuid, Path, description = "Entity ID")
    ),
    responses(
        (status = 204, description = "Entity deleted successfully"),
        (status = 404, description = "Entity not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/entities"
)]
pub async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = EntityRepositoryImpl::new(&state.db);
    repo.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
