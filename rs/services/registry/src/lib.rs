pub mod api;
pub mod config;
pub mod domain;
pub mod dto;
pub mod repository;

use std::sync::Arc;

use chain::ChainClient;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub chain: Arc<ChainClient>,
    pub http_client: reqwest::Client,
    pub config: Arc<config::Config>,
}
