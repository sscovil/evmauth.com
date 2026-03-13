use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::OperatorGrant;

use super::error::RepositoryError;

#[async_trait]
pub trait OperatorGrantRepository: Send + Sync {
    async fn create(
        &self,
        contract_id: Uuid,
        grant_tx_hash: &str,
    ) -> Result<OperatorGrant, RepositoryError>;

    async fn get_by_contract_id(
        &self,
        contract_id: Uuid,
    ) -> Result<Option<OperatorGrant>, RepositoryError>;

    async fn revoke(
        &self,
        id: Uuid,
        revoke_tx_hash: &str,
    ) -> Result<OperatorGrant, RepositoryError>;
}

pub struct OperatorGrantRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> OperatorGrantRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> OperatorGrantRepository for OperatorGrantRepositoryImpl<'a> {
    async fn create(
        &self,
        contract_id: Uuid,
        grant_tx_hash: &str,
    ) -> Result<OperatorGrant, RepositoryError> {
        let result = sqlx::query_as!(
            OperatorGrant,
            r#"
            INSERT INTO registry.operator_grants (contract_id, grant_tx_hash)
            VALUES ($1, $2)
            RETURNING id, contract_id, grant_tx_hash, revoke_tx_hash, active, granted_at, revoked_at, created_at, updated_at
            "#,
            contract_id,
            grant_tx_hash,
        )
        .fetch_one(self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_contract_id(
        &self,
        contract_id: Uuid,
    ) -> Result<Option<OperatorGrant>, RepositoryError> {
        let grant = sqlx::query_as!(
            OperatorGrant,
            r#"
            SELECT id, contract_id, grant_tx_hash, revoke_tx_hash, active, granted_at, revoked_at, created_at, updated_at
            FROM registry.operator_grants
            WHERE contract_id = $1 AND active = true
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            contract_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(grant)
    }

    async fn revoke(
        &self,
        id: Uuid,
        revoke_tx_hash: &str,
    ) -> Result<OperatorGrant, RepositoryError> {
        let result = sqlx::query_as!(
            OperatorGrant,
            r#"
            UPDATE registry.operator_grants
            SET active = false, revoke_tx_hash = $2, revoked_at = now()
            WHERE id = $1
            RETURNING id, contract_id, grant_tx_hash, revoke_tx_hash, active, granted_at, revoked_at, created_at, updated_at
            "#,
            id,
            revoke_tx_hash,
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(result)
    }
}
