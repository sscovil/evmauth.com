use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::response::ContractResponse;
use crate::repository::contract::{ContractRepository, ContractRepositoryImpl};

/// Look up a contract by ID (internal endpoint)
///
/// Used by other services to retrieve contract metadata.
#[utoipa::path(
    get,
    path = "/internal/contracts/{id}",
    params(
        ("id" = Uuid, Path, description = "Contract ID")
    ),
    responses(
        (status = 200, description = "Contract found", body = ContractResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "internal/contracts"
)]
pub async fn get_contract_internal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ContractResponse>, ApiError> {
    let repo = ContractRepositoryImpl::new(&state.db);
    let contract = repo.get(id).await?.ok_or(ApiError::NotFound)?;

    Ok(Json(contract.into()))
}
