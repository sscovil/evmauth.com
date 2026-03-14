use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use pagination::Page;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::AppRegistration;

use super::error::RepositoryError;

/// Generate a random 22-character base64url client ID (128 bits of entropy)
fn generate_client_id() -> Result<String, RepositoryError> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|e| RepositoryError::Internal(format!("failed to generate random bytes: {e}")))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

#[async_trait]
pub trait AppRegistrationRepository: Send + Sync {
    async fn create(
        &self,
        org_id: Uuid,
        name: &str,
        callback_urls: &[String],
        relevant_token_ids: &[i64],
    ) -> Result<AppRegistration, RepositoryError>;

    async fn get(&self, id: Uuid) -> Result<Option<AppRegistration>, RepositoryError>;

    async fn get_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<AppRegistration>, RepositoryError>;

    async fn list_by_org_id(
        &self,
        org_id: Uuid,
        page: &Page,
    ) -> Result<Vec<AppRegistration>, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        callback_urls: Option<&[String]>,
        relevant_token_ids: Option<&[i64]>,
    ) -> Result<AppRegistration, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct AppRegistrationRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> AppRegistrationRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> AppRegistrationRepository for AppRegistrationRepositoryImpl<'a> {
    async fn create(
        &self,
        org_id: Uuid,
        name: &str,
        callback_urls: &[String],
        relevant_token_ids: &[i64],
    ) -> Result<AppRegistration, RepositoryError> {
        let client_id = generate_client_id()?;

        let result = sqlx::query_as!(
            AppRegistration,
            r#"
            INSERT INTO registry.app_registrations (org_id, name, client_id, callback_urls, relevant_token_ids)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, org_id, name, client_id, callback_urls, relevant_token_ids, created_at, updated_at
            "#,
            org_id,
            name,
            client_id,
            callback_urls,
            relevant_token_ids,
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::ConstraintViolation(
                    "Client ID collision; please retry".to_string(),
                );
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<AppRegistration>, RepositoryError> {
        let reg = sqlx::query_as!(
            AppRegistration,
            r#"
            SELECT id, org_id, name, client_id, callback_urls, relevant_token_ids, created_at, updated_at
            FROM registry.app_registrations
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(reg)
    }

    async fn get_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<AppRegistration>, RepositoryError> {
        let reg = sqlx::query_as!(
            AppRegistration,
            r#"
            SELECT id, org_id, name, client_id, callback_urls, relevant_token_ids, created_at, updated_at
            FROM registry.app_registrations
            WHERE client_id = $1
            "#,
            client_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(reg)
    }

    async fn list_by_org_id(
        &self,
        org_id: Uuid,
        page: &Page,
    ) -> Result<Vec<AppRegistration>, RepositoryError> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT id, org_id, name, client_id, callback_urls, relevant_token_ids, created_at, updated_at FROM registry.app_registrations WHERE org_id = ",
        );
        query.push_bind(org_id);

        pagination::apply_cursor_pagination(&mut query, page, None, None)?;

        let mut results = query
            .build_query_as::<AppRegistration>()
            .fetch_all(self.pool)
            .await?;

        pagination::reverse_if_backward(&mut results, page);

        Ok(results)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        callback_urls: Option<&[String]>,
        relevant_token_ids: Option<&[i64]>,
    ) -> Result<AppRegistration, RepositoryError> {
        let result = sqlx::query_as!(
            AppRegistration,
            r#"
            UPDATE registry.app_registrations
            SET
                name = COALESCE($2, name),
                callback_urls = COALESCE($3, callback_urls),
                relevant_token_ids = COALESCE($4, relevant_token_ids)
            WHERE id = $1
            RETURNING id, org_id, name, client_id, callback_urls, relevant_token_ids, created_at, updated_at
            "#,
            id,
            name,
            callback_urls,
            relevant_token_ids,
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM registry.app_registrations
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
