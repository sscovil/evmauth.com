use async_trait::async_trait;
use sqlx::PgPool;
use types::TxHash;
use uuid::Uuid;

use crate::domain::RoleGrant;

use super::error::RepositoryError;

#[async_trait]
pub trait RoleGrantRepository: Send + Sync {
    async fn create(
        &self,
        contract_id: Uuid,
        role: &str,
        grant_tx_hash: &TxHash,
    ) -> Result<RoleGrant, RepositoryError>;

    async fn get_active_by_contract_and_role(
        &self,
        contract_id: Uuid,
        role: &str,
    ) -> Result<Option<RoleGrant>, RepositoryError>;

    async fn list_by_contract_id(
        &self,
        contract_id: Uuid,
    ) -> Result<Vec<RoleGrant>, RepositoryError>;

    async fn revoke(&self, id: Uuid, revoke_tx_hash: &TxHash)
    -> Result<RoleGrant, RepositoryError>;
}

pub struct RoleGrantRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> RoleGrantRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> RoleGrantRepository for RoleGrantRepositoryImpl<'a> {
    async fn create(
        &self,
        contract_id: Uuid,
        role: &str,
        grant_tx_hash: &TxHash,
    ) -> Result<RoleGrant, RepositoryError> {
        let result = sqlx::query_as!(
            RoleGrant,
            r#"
            INSERT INTO registry.role_grants (contract_id, role, grant_tx_hash)
            VALUES ($1, $2, $3)
            RETURNING id, contract_id, role, grant_tx_hash, revoke_tx_hash as "revoke_tx_hash: _", active, granted_at, revoked_at, created_at, updated_at
            "#,
            contract_id,
            role,
            grant_tx_hash.as_str(),
        )
        .fetch_one(self.pool)
        .await?;

        Ok(result)
    }

    async fn get_active_by_contract_and_role(
        &self,
        contract_id: Uuid,
        role: &str,
    ) -> Result<Option<RoleGrant>, RepositoryError> {
        let grant = sqlx::query_as!(
            RoleGrant,
            r#"
            SELECT id, contract_id, role, grant_tx_hash, revoke_tx_hash as "revoke_tx_hash: _", active, granted_at, revoked_at, created_at, updated_at
            FROM registry.role_grants
            WHERE contract_id = $1 AND role = $2 AND active = true
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            contract_id,
            role
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(grant)
    }

    async fn list_by_contract_id(
        &self,
        contract_id: Uuid,
    ) -> Result<Vec<RoleGrant>, RepositoryError> {
        let grants = sqlx::query_as!(
            RoleGrant,
            r#"
            SELECT id, contract_id, role, grant_tx_hash, revoke_tx_hash as "revoke_tx_hash: _", active, granted_at, revoked_at, created_at, updated_at
            FROM registry.role_grants
            WHERE contract_id = $1
            ORDER BY created_at DESC
            "#,
            contract_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(grants)
    }

    async fn revoke(
        &self,
        id: Uuid,
        revoke_tx_hash: &TxHash,
    ) -> Result<RoleGrant, RepositoryError> {
        let result = sqlx::query_as!(
            RoleGrant,
            r#"
            UPDATE registry.role_grants
            SET active = false, revoke_tx_hash = $2, revoked_at = now()
            WHERE id = $1
            RETURNING id, contract_id, role, grant_tx_hash, revoke_tx_hash as "revoke_tx_hash: _", active, granted_at, revoked_at, created_at, updated_at
            "#,
            id,
            revoke_tx_hash.as_str(),
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(result)
    }
}
