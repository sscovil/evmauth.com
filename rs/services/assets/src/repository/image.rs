use async_trait::async_trait;
use pagination::Page;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::Image;
use crate::dto::request::{CreateImage, UpdateImage};

use super::error::RepositoryError;
use super::filter::ImageFilter;

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn create(&self, image: CreateImage) -> Result<Image, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<Image>, RepositoryError>;
    async fn list(&self, filter: ImageFilter, page: Page) -> Result<Vec<Image>, RepositoryError>;
    async fn update(&self, id: Uuid, update: UpdateImage) -> Result<Image, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<Image, RepositoryError>;
}

pub struct ImageRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> ImageRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ImageRepository for ImageRepositoryImpl<'_> {
    async fn create(&self, image: CreateImage) -> Result<Image, RepositoryError> {
        let result = sqlx::query_as::<_, Image>(
            r#"
            INSERT INTO assets.images (org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, created_at, updated_at
            "#,
        )
        .bind(image.org_id)
        .bind(image.uploader_id)
        .bind(&image.object_key)
        .bind(&image.file_name)
        .bind(&image.mime_type)
        .bind(image.size_bytes)
        .bind(image.height)
        .bind(image.width)
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

    async fn get(&self, id: Uuid) -> Result<Option<Image>, RepositoryError> {
        let image = sqlx::query_as::<_, Image>(
            r#"
            SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, created_at, updated_at
            FROM assets.images
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(image)
    }

    async fn list(&self, filter: ImageFilter, page: Page) -> Result<Vec<Image>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, created_at, updated_at FROM assets.images WHERE 1=1",
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

        if let Some(min_width) = filter.min_width {
            query_builder.push(" AND width >= ");
            query_builder.push_bind(min_width);
        }

        if let Some(min_height) = filter.min_height {
            query_builder.push(" AND height >= ");
            query_builder.push_bind(min_height);
        }

        pagination::apply_cursor_pagination(&mut query_builder, &page, None, None)?;

        let mut images = query_builder
            .build_query_as::<Image>()
            .fetch_all(self.pool)
            .await?;

        pagination::reverse_if_backward(&mut images, &page);

        Ok(images)
    }

    async fn update(&self, id: Uuid, update: UpdateImage) -> Result<Image, RepositoryError> {
        let current = self.get(id).await?.ok_or(RepositoryError::NotFound)?;

        let file_name = update.file_name.unwrap_or(current.file_name);

        let image = sqlx::query_as::<_, Image>(
            r#"
            UPDATE assets.images
            SET file_name = $1
            WHERE id = $2
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, created_at, updated_at
            "#,
        )
        .bind(&file_name)
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        Ok(image)
    }

    async fn delete(&self, id: Uuid) -> Result<Image, RepositoryError> {
        let image = sqlx::query_as::<_, Image>(
            r#"
            DELETE FROM assets.images
            WHERE id = $1
            RETURNING id, org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(image)
    }
}
