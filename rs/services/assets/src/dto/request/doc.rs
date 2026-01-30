use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Internal DTO for creating a document record
#[derive(Debug, Clone)]
pub struct CreateDoc {
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub object_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
}

/// Request body for updating document metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateDoc {
    /// New file name
    #[schema(example = "contract.pdf", format = "string")]
    pub file_name: Option<String>,
}
