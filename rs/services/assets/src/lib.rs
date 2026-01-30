pub mod api;
pub mod config;
pub mod domain;
pub mod dto;
pub mod repository;
pub mod s3;

use redis::aio::ConnectionManager;
use s3::S3Client;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub s3: S3Client,
}
