use async_trait::async_trait;
use pagination::Page;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::Media;
use crate::dto::request::{CreateMedia, UpdateMedia};

use super::error::RepositoryError;
use super::filter::MediaFilter;

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn create(&self, media: CreateMedia) -> Result<Media, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<Media>, RepositoryError>;
    async fn list(&self, filter: MediaFilter, page: Page) -> Result<Vec<Media>, RepositoryError>;
    async fn update(&self, id: Uuid, update: UpdateMedia) -> Result<Media, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<Media, RepositoryError>;
}

pub struct MediaRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> MediaRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MediaRepository for MediaRepositoryImpl<'_> {
    async fn create(&self, media: CreateMedia) -> Result<Media, RepositoryError> {
        let result = sqlx::query_as::<_, Media>(
            r#"
            INSERT INTO assets.media (org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, duration_ms, created_at, updated_at
            "#,
        )
        .bind(media.org_id)
        .bind(media.uploader_id)
        .bind(&media.object_key)
        .bind(&media.file_name)
        .bind(&media.mime_type)
        .bind(media.size_bytes)
        .bind(media.height)
        .bind(media.width)
        .bind(media.duration_ms)
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

    async fn get(&self, id: Uuid) -> Result<Option<Media>, RepositoryError> {
        let media = sqlx::query_as::<_, Media>(
            r#"
            SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, duration_ms, created_at, updated_at
            FROM assets.media
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(media)
    }

    async fn list(&self, filter: MediaFilter, page: Page) -> Result<Vec<Media>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, duration_ms, created_at, updated_at FROM assets.media WHERE 1=1",
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

        if let Some(min_duration) = filter.min_duration_ms {
            query_builder.push(" AND duration_ms >= ");
            query_builder.push_bind(min_duration);
        }

        if let Some(max_duration) = filter.max_duration_ms {
            query_builder.push(" AND duration_ms <= ");
            query_builder.push_bind(max_duration);
        }

        pagination::apply_cursor_pagination(&mut query_builder, &page, None, None)?;

        let mut media = query_builder
            .build_query_as::<Media>()
            .fetch_all(self.pool)
            .await?;

        pagination::reverse_if_backward(&mut media, &page);

        Ok(media)
    }

    async fn update(&self, id: Uuid, update: UpdateMedia) -> Result<Media, RepositoryError> {
        let current = self.get(id).await?.ok_or(RepositoryError::NotFound)?;

        let file_name = update.file_name.unwrap_or(current.file_name);

        let media = sqlx::query_as::<_, Media>(
            r#"
            UPDATE assets.media
            SET file_name = $1
            WHERE id = $2
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, duration_ms, created_at, updated_at
            "#,
        )
        .bind(&file_name)
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(media)
    }

    async fn delete(&self, id: Uuid) -> Result<Media, RepositoryError> {
        let media = sqlx::query_as::<_, Media>(
            r#"
            DELETE FROM assets.media
            WHERE id = $1
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, duration_ms, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(media)
    }
}
