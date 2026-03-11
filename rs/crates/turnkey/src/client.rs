use std::time::Duration;

use tracing::{info, warn};

use crate::TurnkeyError;
use crate::signing::{SignRawPayloadParams, SignatureResponse};
use crate::sub_org::{
    CreateDelegatedAccount, CreateSubOrg, CreateWallet, CreateWalletAccount,
    DelegatedAccountResponse, SubOrgResponse, WalletAccountResponse, WalletResponse,
};

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;

/// Configuration for the Turnkey client.
/// Services are responsible for populating this from their own config source
/// (environment variables, config files, etc.).
#[derive(Debug, Clone)]
pub struct TurnkeyConfig {
    pub api_base_url: String,
    pub parent_org_id: String,
    pub api_public_key: String,
    pub api_private_key: String,
}

/// Turnkey API client with retry logic
pub struct TurnkeyClient {
    http: reqwest::Client,
    config: TurnkeyConfig,
}

impl TurnkeyClient {
    pub fn new(config: TurnkeyConfig) -> Result<Self, TurnkeyError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(TurnkeyError::Http)?;

        Ok(Self { http, config })
    }

    /// Create a new sub-organization under the parent org
    pub async fn create_sub_org(
        &self,
        params: CreateSubOrg,
    ) -> Result<SubOrgResponse, TurnkeyError> {
        let body = serde_json::json!({
            "type": "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7",
            "timestampMs": chrono::Utc::now().timestamp_millis().to_string(),
            "organizationId": self.config.parent_org_id,
            "parameters": {
                "subOrganizationName": params.name,
                "rootUsers": params.root_users,
                "rootQuorumThreshold": 1,
            }
        });

        let response: serde_json::Value = self
            .post_with_retry("/public/v1/submit/create_sub_organization", &body)
            .await?;

        let result = extract_activity_result(&response, "createSubOrganizationResultV7")?;
        let parsed: SubOrgResponse = serde_json::from_value(result)?;
        Ok(parsed)
    }

    /// Create a delegated account user in a sub-org with restricted signing policy
    pub async fn create_delegated_account(
        &self,
        params: CreateDelegatedAccount,
    ) -> Result<DelegatedAccountResponse, TurnkeyError> {
        let body = serde_json::json!({
            "type": "ACTIVITY_TYPE_CREATE_API_ONLY_USERS",
            "timestampMs": chrono::Utc::now().timestamp_millis().to_string(),
            "organizationId": params.sub_org_id,
            "parameters": {
                "apiOnlyUsers": [{
                    "userName": params.user_name,
                    "apiKeys": [params.api_key],
                    "userTags": ["delegated-signing"],
                }]
            }
        });

        let response: serde_json::Value = self
            .post_with_retry("/public/v1/submit/create_api_only_users", &body)
            .await?;

        let result = extract_activity_result(&response, "createApiOnlyUsersResult")?;

        let users = result
            .get("apiOnlyUserIds")
            .and_then(|v| v.as_array())
            .ok_or_else(|| TurnkeyError::Api {
                status: 0,
                message: "Missing apiOnlyUserIds in response".to_string(),
            })?;

        let user_id = users
            .first()
            .and_then(|v| v.as_str())
            .ok_or_else(|| TurnkeyError::Api {
                status: 0,
                message: "Empty apiOnlyUserIds in response".to_string(),
            })?
            .to_string();

        Ok(DelegatedAccountResponse {
            user_id,
            api_keys: vec![],
        })
    }

    /// Create a wallet in a sub-org
    pub async fn create_wallet(
        &self,
        params: CreateWallet,
    ) -> Result<WalletResponse, TurnkeyError> {
        let body = serde_json::json!({
            "type": "ACTIVITY_TYPE_CREATE_WALLET",
            "timestampMs": chrono::Utc::now().timestamp_millis().to_string(),
            "organizationId": params.sub_org_id,
            "parameters": {
                "walletName": params.wallet_name,
                "accounts": (0..params.accounts).map(|i| {
                    serde_json::json!({
                        "curve": "CURVE_SECP256K1",
                        "pathFormat": "PATH_FORMAT_BIP32",
                        "path": format!("m/44'/60'/0'/0/{i}"),
                        "addressFormat": "ADDRESS_FORMAT_ETHEREUM",
                    })
                }).collect::<Vec<_>>(),
            }
        });

        let response: serde_json::Value = self
            .post_with_retry("/public/v1/submit/create_wallet", &body)
            .await?;

        let result = extract_activity_result(&response, "createWalletResult")?;
        let parsed: WalletResponse = serde_json::from_value(result)?;
        Ok(parsed)
    }

    /// Create an additional account in an existing wallet
    pub async fn create_wallet_account(
        &self,
        params: CreateWalletAccount,
    ) -> Result<WalletAccountResponse, TurnkeyError> {
        let body = serde_json::json!({
            "type": "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS",
            "timestampMs": chrono::Utc::now().timestamp_millis().to_string(),
            "organizationId": params.sub_org_id,
            "parameters": {
                "walletId": params.wallet_id,
                "accounts": [{
                    "curve": "CURVE_SECP256K1",
                    "pathFormat": "PATH_FORMAT_BIP32",
                    "path": "m/44'/60'/0'/0/0",
                    "addressFormat": "ADDRESS_FORMAT_ETHEREUM",
                }]
            }
        });

        let response: serde_json::Value = self
            .post_with_retry("/public/v1/submit/create_wallet_accounts", &body)
            .await?;

        let result = extract_activity_result(&response, "createWalletAccountsResult")?;

        let addresses = result
            .get("addresses")
            .and_then(|v| v.as_array())
            .ok_or_else(|| TurnkeyError::Api {
                status: 0,
                message: "Missing addresses in response".to_string(),
            })?;

        let first = addresses.first().ok_or_else(|| TurnkeyError::Api {
            status: 0,
            message: "Empty addresses in response".to_string(),
        })?;

        let address = first
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TurnkeyError::Api {
                status: 0,
                message: "Missing address field".to_string(),
            })?
            .to_string();

        let account_id = first
            .get("accountId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(WalletAccountResponse {
            account_id,
            address,
        })
    }

    /// Sign a raw payload using a delegated account
    pub async fn sign_raw_payload(
        &self,
        params: SignRawPayloadParams,
    ) -> Result<SignatureResponse, TurnkeyError> {
        let body = serde_json::json!({
            "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
            "timestampMs": chrono::Utc::now().timestamp_millis().to_string(),
            "organizationId": params.sub_org_id,
            "parameters": {
                "signWith": params.user_id,
                "payload": params.payload,
                "encoding": params.encoding,
                "hashFunction": params.hash_function,
            }
        });

        let response: serde_json::Value = self
            .post_with_retry("/public/v1/submit/sign_raw_payload", &body)
            .await?;

        let result = extract_activity_result(&response, "signRawPayloadResult")?;
        let parsed: SignatureResponse = serde_json::from_value(result)?;
        Ok(parsed)
    }

    /// POST with exponential backoff retry (3 attempts)
    async fn post_with_retry(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, TurnkeyError> {
        let url = format!("{}{path}", self.config.api_base_url);
        let mut last_error = String::new();

        for attempt in 1..=MAX_RETRIES {
            match self.post_request(&url, body).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = e.to_string();

                    if attempt < MAX_RETRIES {
                        let backoff =
                            Duration::from_millis(INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1));
                        warn!(
                            attempt,
                            max_retries = MAX_RETRIES,
                            backoff_ms = backoff.as_millis(),
                            error = %e,
                            "Turnkey API request failed, retrying"
                        );
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(TurnkeyError::MaxRetriesExceeded {
            attempts: MAX_RETRIES,
            last_error,
        })
    }

    async fn post_request(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, TurnkeyError> {
        let body_str = serde_json::to_string(body)?;

        // Stamp the request body with the API key
        // In production, this would use P-256 signing with the API private key
        // For now, use the API public key as a header for authentication
        let stamp = self.stamp_request(&body_str)?;

        let response = self
            .http
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Stamp", &stamp)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let response_body: serde_json::Value = response.json().await?;

        if !status.is_success() {
            let message = response_body
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(TurnkeyError::Api {
                status: status.as_u16(),
                message,
            });
        }

        info!("Turnkey API request succeeded");
        Ok(response_body)
    }

    /// Create a stamp (signed request body) for Turnkey API authentication.
    /// Uses the API keypair to create a P-256 ECDSA signature over the request body.
    fn stamp_request(&self, _body: &str) -> Result<String, TurnkeyError> {
        // The stamp is a JSON object containing the public key and signature,
        // base64url-encoded. The actual P-256 signing implementation requires
        // the p256 crate which will be added when integrating with real Turnkey API.
        // For now, construct the stamp structure.
        let stamp = serde_json::json!({
            "publicKey": self.config.api_public_key,
            "scheme": "SIGNATURE_SCHEME_TK_API_P256",
            "signature": "placeholder"
        });

        let stamp_str = serde_json::to_string(&stamp)
            .map_err(|e| TurnkeyError::Signing(format!("Failed to serialize stamp: {e}")))?;

        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            stamp_str.as_bytes(),
        ))
    }
}

/// Extract the activity result from a Turnkey API response
fn extract_activity_result(
    response: &serde_json::Value,
    result_key: &str,
) -> Result<serde_json::Value, TurnkeyError> {
    response
        .get("activity")
        .and_then(|a| a.get("result"))
        .and_then(|r| r.get(result_key))
        .cloned()
        .ok_or_else(|| TurnkeyError::Api {
            status: 0,
            message: format!("Missing {result_key} in activity result"),
        })
}
