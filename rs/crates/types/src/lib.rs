use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum TypeError {
    #[error("invalid Ethereum address: {0}")]
    InvalidAddress(String),
    #[error("invalid transaction hash: {0}")]
    InvalidTxHash(String),
    #[error("{0} cannot be empty")]
    EmptyValue(&'static str),
}

/// EIP-55 checksummed Ethereum address (wallet or contract).
///
/// Validates and normalizes to EIP-55 checksum format on construction.
/// Transparent for serde, sqlx, and OpenAPI schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(transparent)]
#[serde(transparent)]
#[schema(value_type = String, example = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed")]
pub struct ChecksumAddress(String);

impl ChecksumAddress {
    pub fn new(address: &str) -> Result<Self, TypeError> {
        let parsed: alloy::primitives::Address = address
            .parse()
            .map_err(|e| TypeError::InvalidAddress(format!("{e}")))?;
        Ok(Self(parsed.to_checksum(None)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for ChecksumAddress {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for ChecksumAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Transaction hash (0x-prefixed, 64-char lowercase hex).
///
/// Validates format and normalizes to lowercase on construction.
/// Transparent for serde, sqlx, and OpenAPI schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(transparent)]
#[serde(transparent)]
#[schema(
    value_type = String,
    example = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab"
)]
pub struct TxHash(String);

impl TxHash {
    pub fn new(hash: &str) -> Result<Self, TypeError> {
        let hex = hash
            .strip_prefix("0x")
            .ok_or_else(|| TypeError::InvalidTxHash("must start with '0x'".to_string()))?;
        if hex.len() != 64 {
            return Err(TypeError::InvalidTxHash(format!(
                "expected 64 hex chars, got {}",
                hex.len()
            )));
        }
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(TypeError::InvalidTxHash(
                "contains invalid hex characters".to_string(),
            ));
        }
        Ok(Self(format!("0x{}", hex.to_ascii_lowercase())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for TxHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for TxHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Turnkey sub-organization identifier (opaque string).
///
/// Validates non-empty on construction.
/// Transparent for serde, sqlx, and OpenAPI schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(transparent)]
#[serde(transparent)]
#[schema(value_type = String, example = "sub_org_abc123")]
pub struct TurnkeySubOrgId(String);

impl TurnkeySubOrgId {
    pub fn new(id: &str) -> Result<Self, TypeError> {
        if id.is_empty() {
            return Err(TypeError::EmptyValue("TurnkeySubOrgId"));
        }
        Ok(Self(id.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for TurnkeySubOrgId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for TurnkeySubOrgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Application client identifier (22-char base64url, 128 bits of entropy).
///
/// Validates non-empty on construction.
/// Transparent for serde, sqlx, and OpenAPI schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(transparent)]
#[serde(transparent)]
#[schema(value_type = String, example = "aBcDeFgHiJkLmNoPqRsT01")]
pub struct ClientId(String);

impl ClientId {
    pub fn new(id: &str) -> Result<Self, TypeError> {
        if id.is_empty() {
            return Err(TypeError::EmptyValue("ClientId"));
        }
        Ok(Self(id.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
