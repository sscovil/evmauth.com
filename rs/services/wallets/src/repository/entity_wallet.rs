use async_trait::async_trait;
use sqlx::PgPool;
use types::{ChecksumAddress, TurnkeySubOrgId};
use uuid::Uuid;

use crate::domain::EntityWallet;

use super::error::RepositoryError;

#[async_trait]
pub trait EntityWalletRepository: Send + Sync {
    async fn create(
        &self,
        entity_id: Uuid,
        turnkey_sub_org_id: &TurnkeySubOrgId,
        turnkey_wallet_id: &str,
        wallet_address: &ChecksumAddress,
        turnkey_delegated_user_id: Option<&str>,
    ) -> Result<EntityWallet, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<EntityWallet>, RepositoryError>;
    async fn get_by_entity_id(
        &self,
        entity_id: Uuid,
    ) -> Result<Option<EntityWallet>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct EntityWalletRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> EntityWalletRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> EntityWalletRepository for EntityWalletRepositoryImpl<'a> {
    async fn create(
        &self,
        entity_id: Uuid,
        turnkey_sub_org_id: &TurnkeySubOrgId,
        turnkey_wallet_id: &str,
        wallet_address: &ChecksumAddress,
        turnkey_delegated_user_id: Option<&str>,
    ) -> Result<EntityWallet, RepositoryError> {
        let result = sqlx::query_as!(
            EntityWallet,
            r#"
            INSERT INTO wallets.entity_wallets (entity_id, turnkey_sub_org_id, turnkey_wallet_id, wallet_address, turnkey_delegated_user_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, entity_id, turnkey_sub_org_id, turnkey_wallet_id, wallet_address, turnkey_delegated_user_id, created_at, updated_at
            "#,
            entity_id,
            turnkey_sub_org_id.as_str(),
            turnkey_wallet_id,
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
                    "Entity wallet already exists for this entity".to_string(),
                );
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<EntityWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            EntityWallet,
            r#"
            SELECT id, entity_id, turnkey_sub_org_id, turnkey_wallet_id, wallet_address, turnkey_delegated_user_id, created_at, updated_at
            FROM wallets.entity_wallets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn get_by_entity_id(
        &self,
        entity_id: Uuid,
    ) -> Result<Option<EntityWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            EntityWallet,
            r#"
            SELECT id, entity_id, turnkey_sub_org_id, turnkey_wallet_id, wallet_address, turnkey_delegated_user_id, created_at, updated_at
            FROM wallets.entity_wallets
            WHERE entity_id = $1
            "#,
            entity_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM wallets.entity_wallets
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
