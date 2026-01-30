use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::{Org, OrgVisibility};
use crate::dto::request::{CreateOrg, UpdateOrg};

use super::error::RepositoryError;
use super::filter::OrgFilter;
use super::pagination::Page;

#[async_trait]
pub trait OrgRepository: Send + Sync {
    async fn create(&self, org: CreateOrg) -> Result<Org, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<Org>, RepositoryError>;
    async fn list(&self, filter: OrgFilter, page: Page) -> Result<Vec<Org>, RepositoryError>;
    async fn update(&self, id: Uuid, update: UpdateOrg) -> Result<Org, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct OrgRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> OrgRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> OrgRepository for OrgRepositoryImpl<'a> {
    async fn create(&self, org: CreateOrg) -> Result<Org, RepositoryError> {
        let result = sqlx::query_as!(
            Org,
            r#"
            INSERT INTO auth.orgs (display_name, description, owner_id, visibility)
            VALUES ($1, $2, $3, $4)
            RETURNING id, display_name, description, owner_id, visibility as "visibility: OrgVisibility", created_at, updated_at
            "#,
            org.display_name,
            org.description,
            org.owner_id,
            OrgVisibility::Personal as OrgVisibility
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_foreign_key_violation() {
                    return RepositoryError::ConstraintViolation("Invalid owner_id".to_string());
                }
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Org>, RepositoryError> {
        let org = sqlx::query_as!(
            Org,
            r#"
            SELECT id, display_name, description, owner_id, visibility as "visibility: OrgVisibility", created_at, updated_at
            FROM auth.orgs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(org)
    }

    async fn list(&self, filter: OrgFilter, page: Page) -> Result<Vec<Org>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, display_name, description, owner_id, visibility::text, created_at, updated_at FROM auth.orgs WHERE 1=1"
        );

        if let Some(owner_id) = filter.owner_id {
            query_builder.push(" AND owner_id = ");
            query_builder.push_bind(owner_id);
        }

        if let Some(visibility) = filter.visibility {
            query_builder.push(" AND visibility = ");
            query_builder.push_bind(visibility);
        }

        let search_pattern = filter.search.as_ref().map(|s| format!("%{}%", s));
        if let Some(pattern) = &search_pattern {
            query_builder.push(" AND (display_name ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(" OR description ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(")");
        }

        // Apply cursor-based pagination (Relay spec compliant)
        pagination::apply_cursor_pagination(&mut query_builder, &page, None, None)?;

        let mut orgs = query_builder
            .build_query_as::<Org>()
            .fetch_all(self.pool)
            .await?;

        // Reverse results if backward pagination to maintain consistent ordering
        pagination::reverse_if_backward(&mut orgs, &page);

        Ok(orgs)
    }

    async fn update(&self, id: Uuid, update: UpdateOrg) -> Result<Org, RepositoryError> {
        let current = self.get(id).await?.ok_or(RepositoryError::NotFound)?;

        let display_name = update.display_name.unwrap_or(current.display_name);
        let description = update.description.or(current.description);
        let owner_id = update.owner_id.unwrap_or(current.owner_id);
        let visibility = update.visibility.unwrap_or(current.visibility);

        let org = sqlx::query_as!(
            Org,
            r#"
            UPDATE auth.orgs
            SET display_name = $1, description = $2, owner_id = $3, visibility = $4
            WHERE id = $5
            RETURNING id, display_name, description, owner_id, visibility as "visibility: OrgVisibility", created_at, updated_at
            "#,
            display_name,
            description,
            owner_id,
            visibility as OrgVisibility,
            id
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_foreign_key_violation() {
                    return RepositoryError::ConstraintViolation("Invalid owner_id".to_string());
                }
            }
            RepositoryError::Database(e)
        })?;

        Ok(org)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM auth.orgs
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
