use serde::{Deserialize, Serialize};

/// Parameters for creating a new Turnkey sub-organization
#[derive(Debug, Clone, Serialize)]
pub struct CreateSubOrg {
    /// Display name for the sub-org
    pub name: String,
    /// Root users to add to the sub-org
    pub root_users: Vec<RootUser>,
}

/// A root user in a Turnkey sub-org
#[derive(Debug, Clone, Serialize)]
pub struct RootUser {
    pub user_name: String,
    pub api_keys: Vec<ApiKeyParams>,
    pub authenticators: Vec<AuthenticatorParams>,
}

/// API key parameters for a Turnkey user
#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyParams {
    pub api_key_name: String,
    pub public_key: String,
}

/// Authenticator (passkey) parameters
#[derive(Debug, Clone, Serialize)]
pub struct AuthenticatorParams {
    pub authenticator_name: String,
    pub challenge: String,
    pub attestation: serde_json::Value,
}

/// Response from creating a sub-organization
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubOrgResponse {
    pub sub_organization_id: String,
    pub root_users: Option<Vec<CreatedRootUser>>,
}

/// A root user returned from sub-org creation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedRootUser {
    pub user_id: String,
    pub user_name: String,
    pub api_keys: Option<Vec<CreatedApiKey>>,
}

/// API key returned from creation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedApiKey {
    pub api_key_id: String,
}

/// Parameters for creating a delegated account user
#[derive(Debug, Clone, Serialize)]
pub struct CreateDelegatedAccount {
    /// The sub-org to create the delegated account in
    pub sub_org_id: String,
    /// The user name for the delegated account
    pub user_name: String,
    /// API key for the delegated account
    pub api_key: ApiKeyParams,
}

/// Response from creating a delegated account
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatedAccountResponse {
    pub user_id: String,
    pub api_keys: Vec<CreatedApiKey>,
}

/// Signing policy for delegated accounts
#[derive(Debug, Clone, Serialize)]
pub struct SigningPolicy {
    /// Allowed activity types (e.g., ACTIVITY_TYPE_SIGN_RAW_PAYLOAD)
    pub allowed_activities: Vec<String>,
}

/// Parameters for creating a wallet in a sub-org
#[derive(Debug, Clone, Serialize)]
pub struct CreateWallet {
    pub sub_org_id: String,
    pub wallet_name: String,
    /// Number of accounts to derive
    pub accounts: u32,
}

/// Response from wallet creation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletResponse {
    pub wallet_id: String,
    pub addresses: Vec<String>,
}

/// Parameters for creating an HD wallet account
#[derive(Debug, Clone, Serialize)]
pub struct CreateWalletAccount {
    pub sub_org_id: String,
    pub wallet_id: String,
}

/// Response from creating a wallet account
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletAccountResponse {
    pub account_id: String,
    pub address: String,
}
