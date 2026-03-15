use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use types::{ChecksumAddress, TurnkeySubOrgId};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::OrgWallet;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrgWalletResponse {
    /// The unique identifier for the org wallet record
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// The organization ID
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Uuid,

    /// The Turnkey sub-organization ID
    #[schema(example = "sub_org_abc123", format = "string")]
    pub turnkey_sub_org_id: TurnkeySubOrgId,

    /// The Ethereum wallet address
    #[schema(
        example = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
        format = "string"
    )]
    pub wallet_address: ChecksumAddress,

    /// The Turnkey delegated user ID (if configured)
    #[schema(example = "usr_delegated_123", format = "string")]
    pub turnkey_delegated_user_id: Option<String>,

    /// Timestamp when the record was created
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the record was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<OrgWallet> for OrgWalletResponse {
    fn from(wallet: OrgWallet) -> Self {
        Self {
            id: wallet.id,
            org_id: wallet.org_id,
            turnkey_sub_org_id: wallet.turnkey_sub_org_id,
            wallet_address: wallet.wallet_address,
            turnkey_delegated_user_id: wallet.turnkey_delegated_user_id,
            created_at: wallet.created_at,
            updated_at: wallet.updated_at,
        }
    }
}

impl Pageable for OrgWalletResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
