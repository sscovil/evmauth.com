use std::time::Duration;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use pagination::{Page, PaginatedResponse};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::{CreateAppRegistration, UpdateAppRegistration};
use crate::dto::response::AppRegistrationResponse;
use crate::repository::app_registration::{
    AppRegistrationRepository, AppRegistrationRepositoryImpl,
};

const WALLETS_SERVICE_TIMEOUT: Duration = Duration::from_secs(30);

#[utoipa::path(
    post,
    path = "/orgs/{org_id}/apps",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateAppRegistration,
    responses(
        (status = 201, description = "App registration created", body = AppRegistrationResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "app_registrations"
)]
pub async fn create_app_registration(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<CreateAppRegistration>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = AppRegistrationRepositoryImpl::new(&state.db);

    let callback_urls = body.callback_urls.unwrap_or_default();
    let relevant_token_ids = body.relevant_token_ids.unwrap_or_default();

    let reg = repo
        .create(org_id, &body.name, &callback_urls, &relevant_token_ids)
        .await?;

    // Derive entity app wallet for this (org, app) pair
    let wallets_url = &state.config.wallets_service_url;
    let derive_request = serde_json::json!({
        "entity_id": org_id,
        "app_registration_id": reg.id,
    });

    let derive_result = tokio::time::timeout(WALLETS_SERVICE_TIMEOUT, async {
        state
            .http_client
            .post(format!("{wallets_url}/internal/entity-app-wallet"))
            .json(&derive_request)
            .send()
            .await
    })
    .await;

    match derive_result {
        Ok(Ok(resp)) if resp.status().is_success() => {
            tracing::info!(
                org_id = %org_id,
                app_id = %reg.id,
                "Entity app wallet derived for app registration"
            );
        }
        Ok(Ok(resp)) => {
            let status = resp.status();
            let error_body = resp
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            tracing::warn!(
                org_id = %org_id,
                app_id = %reg.id,
                "Entity app wallet derivation failed: {status} {error_body}"
            );
        }
        Ok(Err(e)) => {
            tracing::warn!(
                org_id = %org_id,
                app_id = %reg.id,
                "Entity app wallet derivation request failed: {e}"
            );
        }
        Err(_) => {
            tracing::warn!(
                org_id = %org_id,
                app_id = %reg.id,
                "Entity app wallet derivation timed out"
            );
        }
    }

    let response: AppRegistrationResponse = reg.into();
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/orgs/{org_id}/apps",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("first" = Option<i64>, Query, description = "Number of items (forward)"),
        ("after" = Option<String>, Query, description = "Cursor (forward)"),
        ("last" = Option<i64>, Query, description = "Number of items (backward)"),
        ("before" = Option<String>, Query, description = "Cursor (backward)")
    ),
    responses(
        (status = 200, description = "List of app registrations", body = PaginatedResponse<AppRegistrationResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "app_registrations"
)]
pub async fn list_app_registrations(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(page): Query<Page>,
) -> Result<Json<PaginatedResponse<AppRegistrationResponse>>, ApiError> {
    let repo = AppRegistrationRepositoryImpl::new(&state.db);
    let results = repo.list_by_org_id(org_id, &page).await?;

    let responses: Vec<AppRegistrationResponse> = results.into_iter().map(Into::into).collect();
    Ok(Json(PaginatedResponse::from_page(responses, &page)))
}

#[utoipa::path(
    get,
    path = "/orgs/{org_id}/apps/{app_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("app_id" = Uuid, Path, description = "App registration ID")
    ),
    responses(
        (status = 200, description = "App registration found", body = AppRegistrationResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "app_registrations"
)]
pub async fn get_app_registration(
    State(state): State<AppState>,
    Path((org_id, app_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AppRegistrationResponse>, ApiError> {
    let repo = AppRegistrationRepositoryImpl::new(&state.db);
    let reg = repo.get(app_id).await?.ok_or(ApiError::NotFound)?;

    if reg.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    Ok(Json(reg.into()))
}

#[utoipa::path(
    patch,
    path = "/orgs/{org_id}/apps/{app_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("app_id" = Uuid, Path, description = "App registration ID")
    ),
    request_body = UpdateAppRegistration,
    responses(
        (status = 200, description = "App registration updated", body = AppRegistrationResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "app_registrations"
)]
pub async fn update_app_registration(
    State(state): State<AppState>,
    Path((org_id, app_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateAppRegistration>,
) -> Result<Json<AppRegistrationResponse>, ApiError> {
    // Verify the app belongs to the org
    let repo = AppRegistrationRepositoryImpl::new(&state.db);
    let existing = repo.get(app_id).await?.ok_or(ApiError::NotFound)?;

    if existing.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    let reg = repo
        .update(
            app_id,
            body.name.as_deref(),
            body.callback_urls.as_deref(),
            body.relevant_token_ids.as_deref(),
        )
        .await?;

    Ok(Json(reg.into()))
}

#[utoipa::path(
    delete,
    path = "/orgs/{org_id}/apps/{app_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("app_id" = Uuid, Path, description = "App registration ID")
    ),
    responses(
        (status = 204, description = "App registration deleted"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "app_registrations"
)]
pub async fn delete_app_registration(
    State(state): State<AppState>,
    Path((org_id, app_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let repo = AppRegistrationRepositoryImpl::new(&state.db);

    // Verify the app belongs to the org
    let existing = repo.get(app_id).await?.ok_or(ApiError::NotFound)?;
    if existing.org_id != org_id {
        return Err(ApiError::NotFound);
    }

    repo.delete(app_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
