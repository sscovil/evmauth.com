use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePersonTurnkeyRef {
    /// The person ID to create the Turnkey sub-org for
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub person_id: Uuid,

    /// Display name for the Turnkey sub-organization
    #[schema(example = "user-sub-org", format = "string")]
    pub sub_org_name: String,

    /// Root user name for the sub-organization
    #[schema(example = "root-user", format = "string")]
    pub root_user_name: String,

    /// API key name for the root user (required for API key auth, optional for passkey auth)
    #[schema(example = "default-api-key", format = "string")]
    pub api_key_name: Option<String>,

    /// API public key for the root user (required for API key auth, optional for passkey auth)
    #[schema(example = "04abc123...", format = "string")]
    pub api_public_key: Option<String>,

    /// Passkey authenticators for the root user (alternative to API key auth)
    #[serde(default)]
    pub authenticators: Vec<PasskeyAttestationParam>,
}

/// Passkey attestation data from WebAuthn registration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PasskeyAttestationParam {
    /// Name for this authenticator
    #[schema(example = "my-passkey", format = "string")]
    pub authenticator_name: String,

    /// The challenge used during registration
    #[schema(example = "base64-encoded-challenge", format = "string")]
    pub challenge: String,

    /// The attestation object from WebAuthn
    pub attestation: serde_json::Value,
}
