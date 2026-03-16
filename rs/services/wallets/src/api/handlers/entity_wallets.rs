use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::EntityWalletResponse;
use crate::repository::entity_wallet::{EntityWalletRepository, EntityWalletRepositoryImpl};

/// Get org wallet info by organization ID
///
/// Because orgs are entities, the org_id is the entity_id in entity_wallets.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/wallet",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Org wallet found", body = EntityWalletResponse),
        (status = 404, description = "Org wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "entity_wallets"
)]
pub async fn get_org_wallet(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<EntityWalletResponse>, ApiError> {
    let repo = EntityWalletRepositoryImpl::new(&state.db);
    let wallet = repo
        .get_by_entity_id(org_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(wallet.into()))
}
