use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use types::TurnkeySubOrgId;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::PersonTurnkeyRef;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PersonTurnkeyRefResponse {
    /// The unique identifier for the person-turnkey ref record
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// The person ID
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub person_id: Uuid,

    /// The Turnkey sub-organization ID
    #[schema(example = "sub_org_abc123", format = "string")]
    pub turnkey_sub_org_id: TurnkeySubOrgId,

    /// Timestamp when the record was created
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the record was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<PersonTurnkeyRef> for PersonTurnkeyRefResponse {
    fn from(ref_record: PersonTurnkeyRef) -> Self {
        Self {
            id: ref_record.id,
            person_id: ref_record.person_id,
            turnkey_sub_org_id: ref_record.turnkey_sub_org_id,
            created_at: ref_record.created_at,
            updated_at: ref_record.updated_at,
        }
    }
}

impl Pageable for PersonTurnkeyRefResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
