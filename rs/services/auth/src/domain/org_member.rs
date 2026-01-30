use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrgMember {
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pageable for OrgMember {
    fn cursor_id(&self) -> Uuid {
        self.member_id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
