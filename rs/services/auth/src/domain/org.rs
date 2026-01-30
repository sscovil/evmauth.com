use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[schema(rename_all = "lowercase")]
pub enum OrgVisibility {
    Personal,
    Private,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Org {
    pub id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub visibility: OrgVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pageable for Org {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
