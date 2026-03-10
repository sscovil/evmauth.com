use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::OrgWalletResponse;
use crate::repository::org_wallet::{OrgWalletRepository, OrgWalletRepositoryImpl};

/// Get org wallet info by organization ID
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/wallet",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Org wallet found", body = OrgWalletResponse),
        (status = 404, description = "Org wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "org_wallets"
)]
pub async fn get_org_wallet(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrgWalletResponse>, ApiError> {
    let repo = OrgWalletRepositoryImpl::new(&state.db);
    let wallet = repo
        .get_by_org_id(org_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(wallet.into()))
}
