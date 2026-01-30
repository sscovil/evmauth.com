use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Internal DTO for creating a file record
#[derive(Debug, Clone)]
pub struct CreateFile {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub object_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
}

/// Request body for updating file metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFile {
    /// New file name
    #[schema(example = "document.pdf", format = "string")]
    pub file_name: Option<String>,
}

/// Request body for getting a presigned upload URL
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PresignedUploadRequest {
    /// Organization ID (optional)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Option<Uuid>,

    /// Original file name
    #[schema(example = "document.pdf", format = "string")]
    pub file_name: String,

    /// MIME type of the file
    #[schema(example = "application/pdf", format = "string")]
    pub content_type: String,

    /// Size of the file in bytes
    #[schema(example = 1024000)]
    pub size_bytes: i64,
}
