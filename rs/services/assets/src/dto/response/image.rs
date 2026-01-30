use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Image;

/// Response containing image metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImageResponse {
    /// Unique identifier for the image
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// Organization ID that owns this image
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub org_id: Option<Uuid>,

    /// ID of the user who uploaded this image
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid")]
    pub uploader_id: Option<Uuid>,

    /// Original file name
    #[schema(example = "photo.jpg", format = "string")]
    pub file_name: String,

    /// MIME type of the image
    #[schema(example = "image/jpeg", format = "string")]
    pub mime_type: String,

    /// Size of the image in bytes
    #[schema(example = 512000)]
    pub size_bytes: i64,

    /// Image height in pixels
    #[schema(example = 1080)]
    pub height: i32,

    /// Image width in pixels
    #[schema(example = 1920)]
    pub width: i32,

    /// Timestamp when the image was uploaded
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the image metadata was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<Image> for ImageResponse {
    fn from(image: Image) -> Self {
        Self {
            id: image.id,
            org_id: image.org_id,
            uploader_id: image.uploader_id,
            file_name: image.file_name,
            mime_type: image.mime_type,
            size_bytes: image.size_bytes,
            height: image.height,
            width: image.width,
            created_at: image.created_at,
            updated_at: image.updated_at,
        }
    }
}

impl Pageable for ImageResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
