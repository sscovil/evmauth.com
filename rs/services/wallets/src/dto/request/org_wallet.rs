use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrgWallet {
    /// The organization ID to create the wallet for
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Uuid,

    /// Display name for the Turnkey sub-organization
    #[schema(example = "My Organization Wallet", format = "string")]
    pub sub_org_name: String,

    /// Optional delegated account user name
    #[schema(example = "delegated-signer", format = "string")]
    pub delegated_user_name: Option<String>,

    /// Optional delegated account API public key
    #[schema(example = "04abc123...", format = "string")]
    pub delegated_api_public_key: Option<String>,
}
