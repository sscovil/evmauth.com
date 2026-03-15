use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateRoleGrant {
    /// EVMAuth role name to grant (e.g. "MINTER_ROLE")
    #[schema(example = "MINTER_ROLE")]
    pub role: String,
}
