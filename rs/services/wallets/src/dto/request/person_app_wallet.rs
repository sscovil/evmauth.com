use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePersonAppWallet {
    /// The person ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub person_id: Uuid,

    /// The app registration ID
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub app_registration_id: Uuid,
}
