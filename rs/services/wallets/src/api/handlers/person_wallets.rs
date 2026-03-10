use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::PersonAppWalletResponse;

/// List the authenticated person's app wallets
///
/// Placeholder: requires auth middleware to extract person_id from token.
/// Currently returns an empty list.
#[utoipa::path(
    get,
    path = "/me/wallets",
    responses(
        (status = 200, description = "List of person's app wallets", body = Vec<PersonAppWalletResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "person_wallets"
)]
pub async fn list_my_wallets(
    State(_state): State<AppState>,
) -> Result<Json<Vec<PersonAppWalletResponse>>, ApiError> {
    // TODO: Extract person_id from auth middleware
    // For now, return empty list until auth middleware is integrated
    Ok(Json(vec![]))
}

/// Get a specific app wallet for the authenticated person
///
/// Placeholder: requires auth middleware to extract person_id from token.
/// Currently returns 404.
#[utoipa::path(
    get,
    path = "/me/wallets/{app_id}",
    params(
        ("app_id" = Uuid, Path, description = "App registration ID")
    ),
    responses(
        (status = 200, description = "Person app wallet found", body = PersonAppWalletResponse),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "person_wallets"
)]
pub async fn get_my_wallet(
    State(_state): State<AppState>,
    Path(_app_id): Path<Uuid>,
) -> Result<Json<PersonAppWalletResponse>, ApiError> {
    // TODO: Extract person_id from auth middleware and look up wallet
    // For now, return 404 until auth middleware is integrated
    Err(ApiError::NotFound)
}
