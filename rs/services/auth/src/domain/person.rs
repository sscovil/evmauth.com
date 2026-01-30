use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    pub auth_provider_name: String,
    pub auth_provider_ref: String,
    pub primary_email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pageable for Person {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
