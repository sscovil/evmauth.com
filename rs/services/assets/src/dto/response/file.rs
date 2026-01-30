use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::File;

/// Response containing file metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileResponse {
    /// Unique identifier for the file
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// Organization ID that owns this file
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub org_id: Option<Uuid>,

    /// ID of the user who uploaded this file
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid")]
    pub uploader_id: Option<Uuid>,

    /// Original file name
    #[schema(example = "document.pdf", format = "string")]
    pub file_name: String,

    /// MIME type of the file
    #[schema(example = "application/pdf", format = "string")]
    pub mime_type: String,

    /// Size of the file in bytes
    #[schema(example = 1024000)]
    pub size_bytes: i64,

    /// Timestamp when the file was uploaded
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the file metadata was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<File> for FileResponse {
    fn from(file: File) -> Self {
        Self {
            id: file.id,
            org_id: file.org_id,
            uploader_id: file.uploader_id,
            file_name: file.file_name,
            mime_type: file.mime_type,
            size_bytes: file.size_bytes,
            created_at: file.created_at,
            updated_at: file.updated_at,
        }
    }
}

impl Pageable for FileResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

/// Response after a successful file upload
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadResponse {
    /// File metadata
    #[schema(value_type = FileResponse)]
    pub file: FileResponse,

    /// Presigned download URL (expires in configured time)
    #[schema(example = "https://s3.example.com/bucket/key?...", format = "uri")]
    pub download_url: String,
}

/// Response containing presigned upload URL
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PresignedUploadResponse {
    /// Unique identifier for the file record (created in pending state)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// Object key in S3
    #[schema(example = "org-id/uuid.pdf", format = "string")]
    pub object_key: String,

    /// Presigned upload URL (expires in configured time)
    #[schema(example = "https://s3.example.com/bucket/key?...", format = "uri")]
    pub upload_url: String,
}
