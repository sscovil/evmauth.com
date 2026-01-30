use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePerson {
    /// The person's display name
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: String,

    /// Optional bio for the person
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
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePerson {
    /// The person's display name
    #[schema(example = "Alice Adams", format = "string")]
    pub display_name: Option<String>,

    /// Optional bio for the person
    #[schema(
        example = "Software engineer and open source contributor",
        format = "string"
    )]
    pub description: Option<String>,

    /// The person's primary email address (must be unique per auth provider)
    #[schema(example = "alice.adams@example.com", format = "email")]
    pub primary_email: Option<String>,
}
