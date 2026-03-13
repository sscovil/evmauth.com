use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Contract;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContractResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    #[schema(example = "660e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Uuid,

    #[schema(example = "770e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub app_registration_id: Option<Uuid>,

    #[schema(example = "My Token Contract")]
    pub name: String,

    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub address: String,

    #[schema(example = "421614")]
    pub chain_id: String,

    #[schema(example = "0xabcdef1234567890abcdef1234567890abcdef12")]
    pub beacon_address: String,

    #[schema(example = "0xdeadbeef...")]
    pub deploy_tx_hash: String,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub deployed_at: DateTime<Utc>,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<Contract> for ContractResponse {
    fn from(c: Contract) -> Self {
        Self {
            id: c.id,
            org_id: c.org_id,
            app_registration_id: c.app_registration_id,
            name: c.name,
            address: c.address,
            chain_id: c.chain_id,
            beacon_address: c.beacon_address,
            deploy_tx_hash: c.deploy_tx_hash,
            deployed_at: c.deployed_at,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

impl Pageable for ContractResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
