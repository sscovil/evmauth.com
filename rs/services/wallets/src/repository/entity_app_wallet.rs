use async_trait::async_trait;
use sqlx::PgPool;
use types::ChecksumAddress;
use uuid::Uuid;

use crate::domain::EntityAppWallet;

use super::error::RepositoryError;

#[async_trait]
pub trait EntityAppWalletRepository: Send + Sync {
    async fn create(
        &self,
        entity_id: Uuid,
        app_registration_id: Uuid,
        wallet_address: &ChecksumAddress,
        turnkey_account_id: &str,
    ) -> Result<EntityAppWallet, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<EntityAppWallet>, RepositoryError>;
    async fn get_by_entity_and_app(
        &self,
        entity_id: Uuid,
        app_registration_id: Uuid,
    ) -> Result<Option<EntityAppWallet>, RepositoryError>;
    async fn list_by_entity_id(
        &self,
        entity_id: Uuid,
    ) -> Result<Vec<EntityAppWallet>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct EntityAppWalletRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> EntityAppWalletRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> EntityAppWalletRepository for EntityAppWalletRepositoryImpl<'a> {
    async fn create(
        &self,
        entity_id: Uuid,
        app_registration_id: Uuid,
        wallet_address: &ChecksumAddress,
        turnkey_account_id: &str,
    ) -> Result<EntityAppWallet, RepositoryError> {
        let result = sqlx::query_as!(
            EntityAppWallet,
            r#"
            INSERT INTO wallets.entity_app_wallets (entity_id, app_registration_id, wallet_address, turnkey_account_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, entity_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            "#,
            entity_id,
            app_registration_id,
            wallet_address.as_str(),
            turnkey_account_id
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::ConstraintViolation(
                    "Entity already has a wallet for this app".to_string(),
                );
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<EntityAppWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            EntityAppWallet,
            r#"
            SELECT id, entity_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            FROM wallets.entity_app_wallets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn get_by_entity_and_app(
        &self,
        entity_id: Uuid,
        app_registration_id: Uuid,
    ) -> Result<Option<EntityAppWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            EntityAppWallet,
            r#"
            SELECT id, entity_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            FROM wallets.entity_app_wallets
            WHERE entity_id = $1 AND app_registration_id = $2
            "#,
            entity_id,
            app_registration_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn list_by_entity_id(
        &self,
        entity_id: Uuid,
    ) -> Result<Vec<EntityAppWallet>, RepositoryError> {
        let wallets = sqlx::query_as!(
            EntityAppWallet,
            r#"
            SELECT id, entity_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            FROM wallets.entity_app_wallets
            WHERE entity_id = $1
            ORDER BY created_at ASC
            "#,
            entity_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(wallets)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM wallets.entity_app_wallets
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
