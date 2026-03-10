use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use pagination::{PaginatedResponse, with_pagination};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::{CreatePerson, UpdatePerson};
use crate::dto::response::PersonResponse;
use crate::repository::filter::PersonFilter;
use crate::repository::person::{PersonRepository, PersonRepositoryImpl};

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListPeopleQuery {
    pub email: Option<String>,
    pub auth_provider_name: Option<String>,
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/people",
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
    let repo = PersonRepositoryImpl::new(&state.db);

    let page = query.to_page()?;

    let mut filter = PersonFilter::new();
    if let Some(ref email) = query.email {
        filter = filter.email(email.clone());
    }
    if let Some(ref provider) = query.auth_provider_name {
        filter = filter.auth_provider(provider.clone());
    }
    if let Some(ref search) = query.search {
        filter = filter.search(search.clone());
    }
    let people = repo.list(filter, page.clone()).await?;

    let responses: Vec<PersonResponse> = people.into_iter().map(|p| p.into()).collect();
    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/people/{id}",
    params(
        ("id" = Uuid, Path, description = "Person ID")
    ),
    responses(
        (status = 200, description = "Person found", body = PersonResponse),
        (status = 404, description = "Person not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "people"
)]
pub async fn get_person(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PersonResponse>, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(person.into()))
}

#[utoipa::path(
    post,
    path = "/people",
    request_body = CreatePerson,
    responses(
        (status = 201, description = "Person created successfully", body = PersonResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "people"
)]
pub async fn create_person(
    State(state): State<AppState>,
    Json(create): Json<CreatePerson>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo.create(create).await?;
    Ok((StatusCode::CREATED, Json(PersonResponse::from(person))))
}

#[utoipa::path(
    put,
    path = "/people/{id}",
    params(
        ("id" = Uuid, Path, description = "Person ID")
    ),
    request_body = UpdatePerson,
    responses(
        (status = 200, description = "Person updated successfully", body = PersonResponse),
        (status = 404, description = "Person not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "people"
)]
pub async fn update_person(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdatePerson>,
) -> Result<Json<PersonResponse>, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    let person = repo.update(id, update).await?;
    Ok(Json(person.into()))
}

#[utoipa::path(
    delete,
    path = "/people/{id}",
    params(
        ("id" = Uuid, Path, description = "Person ID")
    ),
    responses(
        (status = 204, description = "Person deleted successfully"),
        (status = 404, description = "Person not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "people"
)]
pub async fn delete_person(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = PersonRepositoryImpl::new(&state.db);
    repo.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
