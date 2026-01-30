use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Internal DTO for creating an image record
#[derive(Debug, Clone)]
pub struct CreateImage {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub object_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub height: i32,
    pub width: i32,
}

/// Request body for updating image metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateImage {
    /// New file name
    #[schema(example = "photo.jpg", format = "string")]
    pub file_name: Option<String>,
}
