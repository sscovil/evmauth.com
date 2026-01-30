use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Internal DTO for creating a media record
#[derive(Debug, Clone)]
pub struct CreateMedia {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub object_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub height: i32,
    pub width: i32,
    pub duration_ms: i32,
}

/// Request body for updating media metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateMedia {
    /// New file name
    #[schema(example = "video.mp4", format = "string")]
    pub file_name: Option<String>,
}
