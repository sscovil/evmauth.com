pub mod api;
pub mod config;
pub mod domain;
pub mod dto;
pub mod repository;

use std::sync::Arc;

use evm::EvmClient;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub turnkey: Arc<turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>>,
    pub turnkey_parent_org_id: String,
    pub evm: Arc<EvmClient>,
}
