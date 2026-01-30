use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::OrgVisibility;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrg {
    /// The organization's display name
    #[schema(example = "Acme Corporation", format = "string")]
    pub display_name: String,

    /// Optional description for the organization
    #[schema(example = "A software development company", format = "string")]
    pub description: Option<String>,

    /// The ID of the person who owns this organization (ownership of default orgs cannot be transferred)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub owner_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrg {
    ///  The organization's display name
    #[schema(example = "Acme Corporation", format = "string")]
    pub display_name: Option<String>,

    /// Optional description for the organization
    #[schema(example = "A software development company", format = "string")]
    pub description: Option<String>,

    /// The ID of the person who owns this organization (ownership of default orgs cannot be transferred)
    #[schema(
        example = "550e8400-e29b-41d4-a716-446655440000",
        format = "uuid",
        format = "uuid"
    )]
    pub owner_id: Option<Uuid>,

    /// Whether the org is `private`, `public`, or a user's `personal` workspace (which is tied to a single user and cannot have additional members)
    #[schema(
        example = "private",
        format = "string",
        pattern = "^(personal|private|public)$"
    )]
    pub visibility: Option<OrgVisibility>,
}
