use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use types::ChecksumAddress;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::PersonAppWallet;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PersonAppWalletResponse {
    /// The unique identifier for the person app wallet record
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// The person ID
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub person_id: Uuid,

    /// The app registration ID
    #[schema(example = "770e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub app_registration_id: Uuid,

    /// The Ethereum wallet address
    #[schema(
        example = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
        format = "string"
    )]
    pub wallet_address: ChecksumAddress,

    /// The Turnkey account ID within the HD wallet
    #[schema(example = "acct_abc123", format = "string")]
    pub turnkey_account_id: String,

    /// Timestamp when the record was created
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the record was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<PersonAppWallet> for PersonAppWalletResponse {
    fn from(wallet: PersonAppWallet) -> Self {
        Self {
            id: wallet.id,
            person_id: wallet.person_id,
            app_registration_id: wallet.app_registration_id,
            wallet_address: wallet.wallet_address,
            turnkey_account_id: wallet.turnkey_account_id,
            created_at: wallet.created_at,
            updated_at: wallet.updated_at,
        }
    }
}

impl Pageable for PersonAppWalletResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
