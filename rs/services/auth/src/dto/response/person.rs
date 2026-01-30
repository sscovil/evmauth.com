use chrono::{DateTime, Utc};
use pagination::Pageable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Person;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PersonResponse {
    /// The unique identifier for the person
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,

    /// The person's display name
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: String,

    /// Optional description or bio for the person
    #[schema(
        example = "Software engineer and open source contributor",
        format = "string"
    )]
    pub description: Option<String>,

    /// The authentication provider
    #[schema(example = "turnkey", format = "string")]
    pub auth_provider_name: String,

    /// The user ID from the authentication provider
    #[schema(example = "usr_abc123xyz", format = "string")]
    pub auth_provider_ref: String,

    /// The person's primary email address (must be unique per auth provider)
    #[schema(example = "alice.adams@example.com", format = "email")]
    pub primary_email: String,

    /// Timestamp when the record was created
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the record was last updated
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

impl From<Person> for PersonResponse {
    fn from(person: Person) -> Self {
        Self {
            id: person.id,
            display_name: person.display_name,
            description: person.description,
            auth_provider_name: person.auth_provider_name,
            auth_provider_ref: person.auth_provider_ref,
            primary_email: person.primary_email,
            created_at: person.created_at,
            updated_at: person.updated_at,
        }
    }
}

impl Pageable for PersonResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
