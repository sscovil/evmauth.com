use async_trait::async_trait;
use sqlx::PgPool;
use types::ChecksumAddress;
use uuid::Uuid;

use crate::domain::PersonAppWallet;

use super::error::RepositoryError;

#[async_trait]
pub trait PersonAppWalletRepository: Send + Sync {
    async fn create(
        &self,
        person_id: Uuid,
        app_registration_id: Uuid,
        wallet_address: &ChecksumAddress,
        turnkey_account_id: &str,
    ) -> Result<PersonAppWallet, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<PersonAppWallet>, RepositoryError>;
    async fn get_by_person_and_app(
        &self,
        person_id: Uuid,
        app_registration_id: Uuid,
    ) -> Result<Option<PersonAppWallet>, RepositoryError>;
    async fn list_by_person_id(
        &self,
        person_id: Uuid,
    ) -> Result<Vec<PersonAppWallet>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct PersonAppWalletRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> PersonAppWalletRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> PersonAppWalletRepository for PersonAppWalletRepositoryImpl<'a> {
    async fn create(
        &self,
        person_id: Uuid,
        app_registration_id: Uuid,
        wallet_address: &ChecksumAddress,
        turnkey_account_id: &str,
    ) -> Result<PersonAppWallet, RepositoryError> {
        let result = sqlx::query_as!(
            PersonAppWallet,
            r#"
            INSERT INTO wallets.person_app_wallets (person_id, app_registration_id, wallet_address, turnkey_account_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, person_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            "#,
            person_id,
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
                    "Person already has a wallet for this app".to_string(),
                );
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<PersonAppWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            PersonAppWallet,
            r#"
            SELECT id, person_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            FROM wallets.person_app_wallets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn get_by_person_and_app(
        &self,
        person_id: Uuid,
        app_registration_id: Uuid,
    ) -> Result<Option<PersonAppWallet>, RepositoryError> {
        let wallet = sqlx::query_as!(
            PersonAppWallet,
            r#"
            SELECT id, person_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            FROM wallets.person_app_wallets
            WHERE person_id = $1 AND app_registration_id = $2
            "#,
            person_id,
            app_registration_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(wallet)
    }

    async fn list_by_person_id(
        &self,
        person_id: Uuid,
    ) -> Result<Vec<PersonAppWallet>, RepositoryError> {
        let wallets = sqlx::query_as!(
            PersonAppWallet,
            r#"
            SELECT id, person_id, app_registration_id, wallet_address, turnkey_account_id, created_at, updated_at
            FROM wallets.person_app_wallets
            WHERE person_id = $1
            ORDER BY created_at ASC
            "#,
            person_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(wallets)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM wallets.person_app_wallets
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
