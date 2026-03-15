use async_trait::async_trait;
use sqlx::PgPool;
use types::{ChecksumAddress, TurnkeySubOrgId};
use uuid::Uuid;

use crate::domain::OrgWallet;

use super::error::RepositoryError;

#[async_trait]
pub trait OrgWalletRepository: Send + Sync {
    async fn create(
        &self,
        org_id: Uuid,
        turnkey_sub_org_id: &TurnkeySubOrgId,
        wallet_address: &ChecksumAddress,
        turnkey_delegated_user_id: Option<&str>,
    ) -> Result<OrgWallet, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<OrgWallet>, RepositoryError>;
    async fn get_by_org_id(&self, org_id: Uuid) -> Result<Option<OrgWallet>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct OrgWalletRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> OrgWalletRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> OrgWalletRepository for OrgWalletRepositoryImpl<'a> {
    async fn create(
        &self,
        org_id: Uuid,
        turnkey_sub_org_id: &TurnkeySubOrgId,
        wallet_address: &ChecksumAddress,
        turnkey_delegated_user_id: Option<&str>,
    ) -> Result<OrgWallet, RepositoryError> {
        let result = sqlx::query_as!(
            OrgWallet,
            r#"
            INSERT INTO wallets.org_wallets (org_id, turnkey_sub_org_id, wallet_address, turnkey_delegated_user_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, org_id, turnkey_sub_org_id, wallet_address, turnkey_delegated_user_id, created_at, updated_at
            "#,
            org_id,
            turnkey_sub_org_id.as_str(),
            wallet_address.as_str(),
            turnkey_delegated_user_id
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::ConstraintViolation(
                    "Org wallet already exists for this organization".to_string(),
                );
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<OrgWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            OrgWallet,
            r#"
            SELECT id, org_id, turnkey_sub_org_id, wallet_address, turnkey_delegated_user_id, created_at, updated_at
            FROM wallets.org_wallets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn get_by_org_id(&self, org_id: Uuid) -> Result<Option<OrgWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            OrgWallet,
            r#"
            SELECT id, org_id, turnkey_sub_org_id, wallet_address, turnkey_delegated_user_id, created_at, updated_at
            FROM wallets.org_wallets
            WHERE org_id = $1
            "#,
            org_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM wallets.org_wallets
            WHERE id = $1
            "#,
            id
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
