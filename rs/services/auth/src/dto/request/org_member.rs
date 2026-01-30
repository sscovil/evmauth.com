use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrgMember {
    /// The ID of the organization to add the person to
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub member_id: Uuid,

    /// The role assigned to the new person in the organization
    #[schema(
        example = "member",
        format = "string",
        pattern = "^(owner|admin|member)$"
    )]
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrgMember {
    /// The person's role in the organization
    #[schema(
        example = "member",
        format = "string",
        pattern = "^(owner|admin|member)$"
    )]
    pub role: String,
}
