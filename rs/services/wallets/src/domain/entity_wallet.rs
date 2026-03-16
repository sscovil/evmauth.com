use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use types::{ChecksumAddress, TurnkeySubOrgId};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EntityWallet {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub turnkey_sub_org_id: TurnkeySubOrgId,
    pub turnkey_wallet_id: String,
    pub wallet_address: ChecksumAddress,
    pub turnkey_delegated_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pageable for EntityWallet {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
