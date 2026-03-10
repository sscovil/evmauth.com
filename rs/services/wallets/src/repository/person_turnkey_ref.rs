use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::PersonTurnkeyRef;

use super::error::RepositoryError;

#[async_trait]
pub trait PersonTurnkeyRefRepository: Send + Sync {
    async fn create(
        &self,
        person_id: Uuid,
        turnkey_sub_org_id: &str,
    ) -> Result<PersonTurnkeyRef, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<PersonTurnkeyRef>, RepositoryError>;
    async fn get_by_person_id(
        &self,
        person_id: Uuid,
    ) -> Result<Option<PersonTurnkeyRef>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct PersonTurnkeyRefRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> PersonTurnkeyRefRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> PersonTurnkeyRefRepository for PersonTurnkeyRefRepositoryImpl<'a> {
    async fn create(
        &self,
        person_id: Uuid,
        turnkey_sub_org_id: &str,
    ) -> Result<PersonTurnkeyRef, RepositoryError> {
        let result = sqlx::query_as!(
            PersonTurnkeyRef,
            r#"
            INSERT INTO wallets.person_turnkey_refs (person_id, turnkey_sub_org_id)
            VALUES ($1, $2)
            RETURNING id, person_id, turnkey_sub_org_id, created_at, updated_at
            "#,
            person_id,
            turnkey_sub_org_id
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::ConstraintViolation(
                    "Person already has a Turnkey sub-org".to_string(),
                );
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<PersonTurnkeyRef>, RepositoryError> {
        let ref_record = sqlx::query_as!(
            PersonTurnkeyRef,
            r#"
            SELECT id, person_id, turnkey_sub_org_id, created_at, updated_at
            FROM wallets.person_turnkey_refs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(ref_record)
    }

    async fn get_by_person_id(
        &self,
        person_id: Uuid,
    ) -> Result<Option<PersonTurnkeyRef>, RepositoryError> {
        let ref_record = sqlx::query_as!(
            PersonTurnkeyRef,
            r#"
            SELECT id, person_id, turnkey_sub_org_id, created_at, updated_at
            FROM wallets.person_turnkey_refs
            WHERE person_id = $1
            "#,
            person_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(ref_record)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM wallets.person_turnkey_refs
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
