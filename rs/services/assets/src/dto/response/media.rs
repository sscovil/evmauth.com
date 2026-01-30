use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Media;

/// Response containing media (audio/video) metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaResponse {
    /// Unique identifier for the media file
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// Organization ID that owns this media file
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub org_id: Option<Uuid>,

    /// ID of the user who uploaded this media file
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid")]
    pub uploader_id: Option<Uuid>,

    /// Original file name
    #[schema(example = "video.mp4", format = "string")]
    pub file_name: String,

    /// MIME type of the media file
    #[schema(example = "video/mp4", format = "string")]
    pub mime_type: String,

    /// Size of the media file in bytes
    #[schema(example = 10240000)]
    pub size_bytes: i64,

    /// Video height in pixels
    #[schema(example = 1080)]
    pub height: i32,

    /// Video width in pixels
    #[schema(example = 1920)]
    pub width: i32,

    /// Duration in milliseconds
    #[schema(example = 120000)]
    pub duration_ms: i32,

    /// Timestamp when the media file was uploaded
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the media file metadata was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<Media> for MediaResponse {
    fn from(media: Media) -> Self {
        Self {
            id: media.id,
            org_id: media.org_id,
            uploader_id: media.uploader_id,
            file_name: media.file_name,
            mime_type: media.mime_type,
            size_bytes: media.size_bytes,
            height: media.height,
            width: media.width,
            duration_ms: media.duration_ms,
            created_at: media.created_at,
            updated_at: media.updated_at,
        }
    }
}

impl Pageable for MediaResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
