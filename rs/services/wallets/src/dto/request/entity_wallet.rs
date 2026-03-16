use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateEntityWallet {
    /// The entity ID (person or org) to create the wallet for
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub entity_id: Uuid,

    /// Display name for the Turnkey sub-organization
    #[schema(example = "My Entity Wallet", format = "string")]
    pub sub_org_name: String,

    /// Root user name for the sub-organization (required for person entities)
    #[schema(example = "root-user", format = "string")]
    pub root_user_name: Option<String>,

    /// API key name for the root user (optional, for person API key auth)
    #[schema(example = "default-api-key", format = "string")]
    pub api_key_name: Option<String>,

    /// API public key for the root user (optional, for person API key auth)
    #[schema(example = "04abc123...", format = "string")]
    pub api_public_key: Option<String>,

    /// Passkey authenticators for the root user (for person passkey auth)
    #[serde(default)]
    pub authenticators: Vec<PasskeyAttestationParam>,

    /// Optional delegated account user name (for org entities)
    #[schema(example = "delegated-signer", format = "string")]
    pub delegated_user_name: Option<String>,

    /// Optional delegated account API public key (for org entities)
    #[schema(example = "04abc123...", format = "string")]
    pub delegated_api_public_key: Option<String>,
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
