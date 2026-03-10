use serde::{Deserialize, Serialize};

/// Parameters for signing a raw payload
#[derive(Debug, Clone, Serialize)]
pub struct SignRawPayloadParams {
    pub sub_org_id: String,
    pub user_id: String,
    pub payload: String,
    pub encoding: PayloadEncoding,
    pub hash_function: HashFunction,
}

/// Payload encoding format
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayloadEncoding {
    PayloadEncodingHexadecimal,
    PayloadEncodingUtf8,
}

/// Hash function for signing
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HashFunction {
    HashFunctionNoOp,
    HashFunctionSha256,
    HashFunctionKeccak256,
}

/// Signature response
#[derive(Debug, Clone, Deserialize)]
pub struct SignatureResponse {
    pub r: String,
    pub s: String,
    pub v: String,
}
