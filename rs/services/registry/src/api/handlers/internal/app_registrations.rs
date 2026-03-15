use axum::{
    Json,
    extract::{Path, State},
};

use types::ClientId;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::AppRegistrationResponse;
use crate::repository::app_registration::{
    AppRegistrationRepository, AppRegistrationRepositoryImpl,
};

/// Look up an app registration by client_id (internal endpoint)
///
/// Used by the auth service during the PKCE flow to validate client_id
/// and retrieve callback_urls.
#[utoipa::path(
    get,
    path = "/internal/apps/by-client-id/{client_id}",
    params(
        ("client_id" = String, Path, description = "App registration client ID")
    ),
    responses(
        (status = 200, description = "App registration found", body = AppRegistrationResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/app_registrations"
)]
pub async fn get_app_by_client_id(
    State(state): State<AppState>,
    Path(client_id_str): Path<String>,
) -> Result<Json<AppRegistrationResponse>, ApiError> {
    let client_id = ClientId::new(&client_id_str)
        .map_err(|e| ApiError::BadRequest(format!("invalid client ID: {e}")))?;

    let repo = AppRegistrationRepositoryImpl::new(&state.db);
    let reg = repo
        .get_by_client_id(&client_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(reg.into()))
}
