use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Entity;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EntityResponse {
    /// The unique identifier for the person or organization
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// The table this entity belongs to (e.g. 'people', 'orgs')
    #[schema(example = "people", format = "string")]
    pub table: Option<String>,

    /// The display name of the person or organization
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: String,

    /// Optional description or bio for the person or organization
    #[schema(
        example = "Software engineer and open source contributor",
        format = "string"
    )]
    pub description: Option<String>,

    /// Timestamp when the record was created
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the record was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<Entity> for EntityResponse {
    fn from(entity: Entity) -> Self {
        Self {
            id: entity.id,
            table: entity.table,
            display_name: entity.display_name,
            description: entity.description,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

impl Pageable for EntityResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
