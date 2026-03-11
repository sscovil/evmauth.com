pub mod api;
pub mod config;
pub mod domain;
pub mod dto;
pub mod jwt;
pub mod middleware;
pub mod repository;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;
use crate::jwt::JwtKeys;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub jwt_keys: Option<Arc<JwtKeys>>,
    pub http_client: reqwest::Client,
    pub config: Arc<Config>,
    pub chain: Arc<chain::ChainClient>,
}
