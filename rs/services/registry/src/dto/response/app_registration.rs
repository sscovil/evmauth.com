use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use types::ClientId;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::AppRegistration;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppRegistrationResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    #[schema(example = "660e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub org_id: Uuid,

    #[schema(example = "My DApp")]
    pub name: String,

    #[schema(example = "aBcDeFgHiJkLmNoPqRsT01")]
    pub client_id: ClientId,

    #[schema(example = json!(["https://example.com/callback"]))]
    pub callback_urls: Vec<String>,

    #[schema(example = json!([1, 2]))]
    pub relevant_token_ids: Vec<i64>,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<AppRegistration> for AppRegistrationResponse {
    fn from(reg: AppRegistration) -> Self {
        Self {
            id: reg.id,
            org_id: reg.org_id,
            name: reg.name,
            client_id: reg.client_id,
            callback_urls: reg.callback_urls,
            relevant_token_ids: reg.relevant_token_ids,
            created_at: reg.created_at,
            updated_at: reg.updated_at,
        }
    }
}

impl Pageable for AppRegistrationResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
