use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Doc;

/// Response containing document metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocResponse {
    /// Unique identifier for the document
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// Organization ID that owns this document
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub org_id: Option<Uuid>,

    /// ID of the user who uploaded this document
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid")]
    pub uploader_id: Option<Uuid>,

    /// Original file name
    #[schema(example = "contract.pdf", format = "string")]
    pub file_name: String,

    /// MIME type of the document
    #[schema(example = "application/pdf", format = "string")]
    pub mime_type: String,

    /// Size of the document in bytes
    #[schema(example = 2048000)]
    pub size_bytes: i64,

    /// Timestamp when the document was uploaded
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the document metadata was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<Doc> for DocResponse {
    fn from(doc: Doc) -> Self {
        Self {
            id: doc.id,
            org_id: doc.org_id,
            uploader_id: doc.uploader_id,
            file_name: doc.file_name,
            mime_type: doc.mime_type,
            size_bytes: doc.size_bytes,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

impl Pageable for DocResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
