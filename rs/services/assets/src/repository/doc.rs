use async_trait::async_trait;
use pagination::Page;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::Doc;
use crate::dto::request::{CreateDoc, UpdateDoc};

use super::error::RepositoryError;
use super::filter::DocFilter;

#[async_trait]
pub trait DocRepository: Send + Sync {
    async fn create(&self, doc: CreateDoc) -> Result<Doc, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<Doc>, RepositoryError>;
    async fn list(&self, filter: DocFilter, page: Page) -> Result<Vec<Doc>, RepositoryError>;
    async fn update(&self, id: Uuid, update: UpdateDoc) -> Result<Doc, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<Doc, RepositoryError>;
}

pub struct DocRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> DocRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DocRepository for DocRepositoryImpl<'_> {
    async fn create(&self, doc: CreateDoc) -> Result<Doc, RepositoryError> {
        let result = sqlx::query_as::<_, Doc>(
            r#"
            INSERT INTO assets.docs (org_id, uploader_id, object_key, file_name, mime_type, size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            "#,
        )
        .bind(doc.org_id)
        .bind(doc.uploader_id)
        .bind(&doc.object_key)
        .bind(&doc.file_name)
        .bind(&doc.mime_type)
        .bind(doc.size_bytes)
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::ConstraintViolation("Object key already exists".to_string());
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Doc>, RepositoryError> {
        let doc = sqlx::query_as::<_, Doc>(
            r#"
            SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            FROM assets.docs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(doc)
    }

    async fn list(&self, filter: DocFilter, page: Page) -> Result<Vec<Doc>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at FROM assets.docs WHERE 1=1",
        );

        if let Some(org_id) = &filter.org_id {
            query_builder.push(" AND org_id = ");
            query_builder.push_bind(org_id);
        }

        if let Some(uploader_id) = &filter.uploader_id {
            query_builder.push(" AND uploader_id = ");
            query_builder.push_bind(uploader_id);
        }

        if let Some(mime_type) = &filter.mime_type {
            query_builder.push(" AND mime_type = ");
            query_builder.push_bind(mime_type);
        }

        let search_pattern = filter.search.as_ref().map(|s| format!("%{s}%"));
        if let Some(pattern) = &search_pattern {
            query_builder.push(" AND file_name ILIKE ");
            query_builder.push_bind(pattern);
        }

        pagination::apply_cursor_pagination(&mut query_builder, &page, None, None)?;

        let mut docs = query_builder
            .build_query_as::<Doc>()
            .fetch_all(self.pool)
            .await?;

        pagination::reverse_if_backward(&mut docs, &page);

        Ok(docs)
    }

    async fn update(&self, id: Uuid, update: UpdateDoc) -> Result<Doc, RepositoryError> {
        let current = self.get(id).await?.ok_or(RepositoryError::NotFound)?;

        let file_name = update.file_name.unwrap_or(current.file_name);

        let doc = sqlx::query_as::<_, Doc>(
            r#"
            UPDATE assets.docs
            SET file_name = $1
            WHERE id = $2
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            "#,
        )
        .bind(&file_name)
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(doc)
    }

    async fn delete(&self, id: Uuid) -> Result<Doc, RepositoryError> {
        let doc = sqlx::query_as::<_, Doc>(
            r#"
            DELETE FROM assets.docs
            WHERE id = $1
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(doc)
    }
}
