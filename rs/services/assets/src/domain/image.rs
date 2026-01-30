use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Image {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub uploader_id: Option<Uuid>,
    pub object_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub height: i32,
    pub width: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pageable for Image {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
