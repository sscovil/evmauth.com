use async_trait::async_trait;
use pagination::Page;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::File;
use crate::dto::request::{CreateFile, UpdateFile};

use super::error::RepositoryError;
use super::filter::FileFilter;

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn create(&self, file: CreateFile) -> Result<File, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<File>, RepositoryError>;
    async fn list(&self, filter: FileFilter, page: Page) -> Result<Vec<File>, RepositoryError>;
    async fn update(&self, id: Uuid, update: UpdateFile) -> Result<File, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<File, RepositoryError>;
}

pub struct FileRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> FileRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileRepository for FileRepositoryImpl<'_> {
    async fn create(&self, file: CreateFile) -> Result<File, RepositoryError> {
        let result = sqlx::query_as::<_, File>(
            r#"
            INSERT INTO assets.files (org_id, uploader_id, object_key, file_name, mime_type, size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            "#,
        )
        .bind(file.org_id)
        .bind(file.uploader_id)
        .bind(&file.object_key)
        .bind(&file.file_name)
        .bind(&file.mime_type)
        .bind(file.size_bytes)
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

    async fn get(&self, id: Uuid) -> Result<Option<File>, RepositoryError> {
        let file = sqlx::query_as::<_, File>(
            r#"
            SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            FROM assets.files
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(file)
    }

    async fn list(&self, filter: FileFilter, page: Page) -> Result<Vec<File>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at FROM assets.files WHERE 1=1",
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

        let mut files = query_builder
            .build_query_as::<File>()
            .fetch_all(self.pool)
            .await?;

        pagination::reverse_if_backward(&mut files, &page);

        Ok(files)
    }

    async fn update(&self, id: Uuid, update: UpdateFile) -> Result<File, RepositoryError> {
        let current = self.get(id).await?.ok_or(RepositoryError::NotFound)?;

        let file_name = update.file_name.unwrap_or(current.file_name);

        let file = sqlx::query_as::<_, File>(
            r#"
            UPDATE assets.files
            SET file_name = $1
            WHERE id = $2
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            "#,
        )
        .bind(&file_name)
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(file)
    }

    async fn delete(&self, id: Uuid) -> Result<File, RepositoryError> {
        let file = sqlx::query_as::<_, File>(
            r#"
            DELETE FROM assets.files
            WHERE id = $1
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(file)
    }
}
