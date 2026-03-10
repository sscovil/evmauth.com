use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePersonTurnkeyRef {
    /// The person ID to create the Turnkey sub-org for
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub person_id: Uuid,

    /// Display name for the Turnkey sub-organization
    #[schema(example = "user-sub-org", format = "string")]
    pub sub_org_name: String,

    /// Root user name for the sub-organization
    #[schema(example = "root-user", format = "string")]
    pub root_user_name: String,

    /// API key name for the root user
    #[schema(example = "default-api-key", format = "string")]
    pub api_key_name: String,

    /// API public key for the root user
    #[schema(example = "04abc123...", format = "string")]
    pub api_public_key: String,
}
