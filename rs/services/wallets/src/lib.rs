pub mod api;
pub mod config;
pub mod domain;
pub mod dto;
pub mod repository;

use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use turnkey::TurnkeyClient;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub turnkey: Arc<TurnkeyClient>,
}
