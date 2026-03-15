use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use types::TxHash;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OperatorGrant {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub grant_tx_hash: TxHash,
    pub revoke_tx_hash: Option<TxHash>,
    pub active: bool,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
