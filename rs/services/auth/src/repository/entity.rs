use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::Entity;

use super::error::RepositoryError;
use super::filter::EntityFilter;
use super::pagination::Page;

#[async_trait]
pub trait EntityRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Option<Entity>, RepositoryError>;
    async fn list(&self, filter: EntityFilter, page: Page) -> Result<Vec<Entity>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct EntityRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> EntityRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> EntityRepository for EntityRepositoryImpl<'a> {
    async fn get(&self, id: Uuid) -> Result<Option<Entity>, RepositoryError> {
        let entity = sqlx::query_as!(
            Entity,
            r#"
            SELECT
                e.id,
                e.tableoid::regclass::text AS table,
                e.display_name,
                e.description,
                e.created_at,
                e.updated_at
            FROM auth.entities* e
            WHERE e.id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(entity)
    }

    async fn list(&self, filter: EntityFilter, page: Page) -> Result<Vec<Entity>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT e.id, e.tableoid::regclass::text AS table, e.display_name, e.description, e.created_at, e.updated_at FROM auth.entities* e WHERE 1=1",
        );

        if let Some(entity_type) = &filter.entity_type {
            query_builder.push(" AND e.tableoid::regclass::text = ");
            query_builder.push_bind(entity_type);
        }

        let search_pattern = filter.search.as_ref().map(|s| format!("%{}%", s));
        if let Some(pattern) = &search_pattern {
            query_builder.push(" AND (e.display_name ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(" OR e.description ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(")");
        }

        // Apply cursor-based pagination with table alias (Relay spec compliant)
        pagination::apply_cursor_pagination(
            &mut query_builder,
            &page,
            Some("e.id"),
            Some("e.created_at"),
        )?;

        let mut entities = query_builder
            .build_query_as::<Entity>()
            .fetch_all(self.pool)
            .await?;

        // Reverse results if backward pagination to maintain consistent ordering
        pagination::reverse_if_backward(&mut entities, &page);

        Ok(entities)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM auth.entities
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
