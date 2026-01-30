use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{Org, OrgVisibility};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrgResponse {
    /// The unique identifier for the organization
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// The organization's display name
    #[schema(example = "Acme Corporation", format = "string")]
    pub display_name: String,

    /// Optional description for the organization
    #[schema(example = "A software development company", format = "string")]
    pub description: Option<String>,

    /// The ID of the person who owns this organization (ownership of default orgs cannot be transferred)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub owner_id: Uuid,

    /// Whether this is the person's private workspace
    #[schema(example = "private")]
    pub visibility: OrgVisibility,

    /// Timestamp when the record was created
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the record was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<Org> for OrgResponse {
    fn from(org: Org) -> Self {
        Self {
            id: org.id,
            display_name: org.display_name,
            description: org.description,
            owner_id: org.owner_id,
            visibility: org.visibility,
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}

impl Pageable for OrgResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
