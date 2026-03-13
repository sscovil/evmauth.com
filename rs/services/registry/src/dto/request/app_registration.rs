use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAppRegistration {
    /// Display name for the app registration
    #[schema(example = "My DApp")]
    pub name: String,

    /// Allowed OAuth callback URLs
    #[schema(example = json!(["https://example.com/callback"]))]
    pub callback_urls: Option<Vec<String>>,

    /// ERC-6909 token IDs relevant to this app
    #[schema(example = json!([1, 2]))]
    pub relevant_token_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAppRegistration {
    /// Display name for the app registration
    #[schema(example = "My DApp v2")]
    pub name: Option<String>,

    /// Allowed OAuth callback URLs
    #[schema(example = json!(["https://example.com/callback"]))]
    pub callback_urls: Option<Vec<String>>,

    /// ERC-6909 token IDs relevant to this app
    #[schema(example = json!([1, 2, 3]))]
    pub relevant_token_ids: Option<Vec<i64>>,
}
