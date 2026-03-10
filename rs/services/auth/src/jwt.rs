use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to create JWT: {0}")]
    Creation(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

/// RS256 key pair for JWT signing and verification
#[derive(Clone)]
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    /// Create JwtKeys from PEM-encoded RSA private and public keys
    pub fn from_pem(private_pem: &[u8], public_pem: &[u8]) -> Result<Self, JwtError> {
        let encoding = EncodingKey::from_rsa_pem(private_pem)
            .map_err(|e| JwtError::InvalidKey(format!("Invalid private key: {e}")))?;
        let decoding = DecodingKey::from_rsa_pem(public_pem)
            .map_err(|e| JwtError::InvalidKey(format!("Invalid public key: {e}")))?;

        Ok(Self { encoding, decoding })
    }

    /// Get the public decoding key reference
    pub fn public_key_pem(&self) -> &DecodingKey {
        &self.decoding
    }
}

/// Claims for a platform session JWT (deployer dashboard)
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionClaims {
    /// Issuer
    pub iss: String,
    /// Subject (person ID)
    pub sub: String,
    /// Token type
    #[serde(rename = "type")]
    pub token_type: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
}

/// Claims for an end-user app JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct EndUserClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    #[serde(rename = "type")]
    pub token_type: String,
    pub wallet: String,
    pub contract: String,
    pub chain_id: String,
    pub exp: i64,
    pub iat: i64,
}

/// Create a platform session JWT
pub fn create_session_token(
    keys: &JwtKeys,
    person_id: Uuid,
    duration_hours: i64,
) -> Result<String, JwtError> {
    let now = chrono::Utc::now().timestamp();
    let claims = SessionClaims {
        iss: "https://api.evmauth.io".to_string(),
        sub: person_id.to_string(),
        token_type: "session".to_string(),
        exp: now + (duration_hours * 3600),
        iat: now,
    };

    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, &keys.encoding)?;
    Ok(token)
}

/// Verify and decode a session JWT
pub fn verify_session_token(keys: &JwtKeys, token: &str) -> Result<SessionClaims, JwtError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://api.evmauth.io"]);
    validation.validate_exp = true;

    let token_data = decode::<SessionClaims>(token, &keys.decoding, &validation)?;
    Ok(token_data.claims)
}

/// Create an end-user app JWT
pub fn create_end_user_token(
    keys: &JwtKeys,
    person_id: Uuid,
    client_id: &str,
    wallet_address: &str,
    contract_address: &str,
    chain_id: &str,
    duration_secs: i64,
) -> Result<String, JwtError> {
    let now = chrono::Utc::now().timestamp();
    let claims = EndUserClaims {
        iss: "https://auth.evmauth.io".to_string(),
        sub: person_id.to_string(),
        aud: client_id.to_string(),
        token_type: "end_user".to_string(),
        wallet: wallet_address.to_string(),
        contract: contract_address.to_string(),
        chain_id: chain_id.to_string(),
        exp: now + duration_secs,
        iat: now,
    };

    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, &keys.encoding)?;
    Ok(token)
}
