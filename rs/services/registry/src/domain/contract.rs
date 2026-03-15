use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use types::{ChecksumAddress, TxHash};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contract {
    pub id: Uuid,
    pub org_id: Uuid,
    pub app_registration_id: Option<Uuid>,
    pub name: String,
    pub address: ChecksumAddress,
    pub chain_id: String,
    pub beacon_address: ChecksumAddress,
    pub deploy_tx_hash: TxHash,
    pub deployed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pageable for Contract {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
