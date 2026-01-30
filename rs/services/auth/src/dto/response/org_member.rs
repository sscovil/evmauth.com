use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::OrgMember;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrgMemberResponse {
    /// The ID of the organization the person belongs to
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Uuid,

    /// The ID of the person who is a member of the organization
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub member_id: Uuid,

    /// The member's role in the organization
    #[schema(
        example = "member",
        format = "string",
        pattern = "^(owner|admin|member)$"
    )]
    pub role: String,

    /// Timestamp when the record was created
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the record was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<OrgMember> for OrgMemberResponse {
    fn from(member: OrgMember) -> Self {
        Self {
            org_id: member.org_id,
            member_id: member.member_id,
            role: member.role,
            created_at: member.created_at,
            updated_at: member.updated_at,
        }
    }
}

impl Pageable for OrgMemberResponse {
    fn cursor_id(&self) -> Uuid {
        self.member_id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
