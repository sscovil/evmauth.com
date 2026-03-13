use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateContract {
    /// Display name for the contract
    #[schema(example = "My Token Contract")]
    pub name: String,

    /// Optional app registration to associate with this contract
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub app_registration_id: Option<Uuid>,
}
