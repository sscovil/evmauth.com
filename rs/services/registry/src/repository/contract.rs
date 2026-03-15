use async_trait::async_trait;
use pagination::Page;
use sqlx::{PgPool, Postgres, QueryBuilder};
use types::{ChecksumAddress, TxHash};
use uuid::Uuid;

use crate::domain::Contract;

use super::error::RepositoryError;

pub struct CreateContractParams {
    pub org_id: Uuid,
    pub app_registration_id: Option<Uuid>,
    pub name: String,
    pub address: ChecksumAddress,
    pub chain_id: String,
    pub beacon_address: ChecksumAddress,
    pub deploy_tx_hash: TxHash,
}

#[async_trait]
pub trait ContractRepository: Send + Sync {
    async fn create(&self, params: CreateContractParams) -> Result<Contract, RepositoryError>;

    async fn get(&self, id: Uuid) -> Result<Option<Contract>, RepositoryError>;

    async fn get_by_address(
        &self,
        address: &ChecksumAddress,
    ) -> Result<Option<Contract>, RepositoryError>;

    async fn list_by_org_id(
        &self,
        org_id: Uuid,
        page: &Page,
    ) -> Result<Vec<Contract>, RepositoryError>;
}

pub struct ContractRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> ContractRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> ContractRepository for ContractRepositoryImpl<'a> {
    async fn create(&self, params: CreateContractParams) -> Result<Contract, RepositoryError> {
        let result = sqlx::query_as!(
            Contract,
            r#"
            INSERT INTO registry.contracts (org_id, app_registration_id, name, address, chain_id, beacon_address, deploy_tx_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, app_registration_id, name, address, chain_id, beacon_address, deploy_tx_hash, deployed_at, created_at, updated_at
            "#,
            params.org_id,
            params.app_registration_id,
            params.name,
            params.address.as_str(),
            params.chain_id,
            params.beacon_address.as_str(),
            params.deploy_tx_hash.as_str(),
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::ConstraintViolation(
                    "Contract address already registered".to_string(),
                );
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Contract>, RepositoryError> {
        let contract = sqlx::query_as!(
            Contract,
            r#"
            SELECT id, org_id, app_registration_id, name, address, chain_id, beacon_address, deploy_tx_hash, deployed_at, created_at, updated_at
            FROM registry.contracts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(contract)
    }

    async fn get_by_address(
        &self,
        address: &ChecksumAddress,
    ) -> Result<Option<Contract>, RepositoryError> {
        let contract = sqlx::query_as!(
            Contract,
            r#"
            SELECT id, org_id, app_registration_id, name, address, chain_id, beacon_address, deploy_tx_hash, deployed_at, created_at, updated_at
            FROM registry.contracts
            WHERE address = $1
            "#,
            address.as_str()
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(contract)
    }

    async fn list_by_org_id(
        &self,
        org_id: Uuid,
        page: &Page,
    ) -> Result<Vec<Contract>, RepositoryError> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT id, org_id, app_registration_id, name, address, chain_id, beacon_address, deploy_tx_hash, deployed_at, created_at, updated_at FROM registry.contracts WHERE org_id = ",
        );
        query.push_bind(org_id);

        pagination::apply_cursor_pagination(&mut query, page, None, None)?;

        let mut results = query
            .build_query_as::<Contract>()
            .fetch_all(self.pool)
            .await?;

        pagination::reverse_if_backward(&mut results, page);

        Ok(results)
    }
}
