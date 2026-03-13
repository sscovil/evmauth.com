use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::OperatorGrant;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OperatorGrantResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    #[schema(example = "660e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub contract_id: Uuid,

    #[schema(example = "0xabc123...")]
    pub grant_tx_hash: String,

    #[schema(example = "0xdef456...")]
    pub revoke_tx_hash: Option<String>,

    pub active: bool,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub granted_at: DateTime<Utc>,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub revoked_at: Option<DateTime<Utc>>,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<OperatorGrant> for OperatorGrantResponse {
    fn from(g: OperatorGrant) -> Self {
        Self {
            id: g.id,
            contract_id: g.contract_id,
            grant_tx_hash: g.grant_tx_hash,
            revoke_tx_hash: g.revoke_tx_hash,
            active: g.active,
            granted_at: g.granted_at,
            revoked_at: g.revoked_at,
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}
