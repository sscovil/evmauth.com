use serde::Deserialize;
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub pg: DatabaseConfig,
    pub redis: redis_client::RedisConfig,
    pub s3: S3Config,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(flatten)]
    pub config: postgres::PGConfig,
    pub pool: DatabaseConnectionPoolConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConnectionPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub presigned_url_expiry: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let config = Config {
            pg: DatabaseConfig {
                config: postgres::PGConfig {
                    host: env::var("POSTGRES_HOST")?,
                    port: env::var("POSTGRES_PORT")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(5432),
                    user: env::var("POSTGRES_USER")?,
                    password: env::var("POSTGRES_PASSWORD")?,
                    database: env::var("POSTGRES_DB")?,
                },
                pool: DatabaseConnectionPoolConfig {
                    max_connections: env::var("POSTGRES_MAX_CONNECTIONS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(5),
                    min_connections: env::var("POSTGRES_MIN_CONNECTIONS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                },
            },
            redis: redis_client::RedisConfig {
                host: env::var("REDIS_HOST")?,
                port: env::var("REDIS_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(6379),
                password: env::var("REDIS_PASSWORD").ok(),
            },
            s3: S3Config {
                bucket: env::var("S3_BUCKET")?,
                region: env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string()),
                endpoint: env::var("S3_ENDPOINT").ok(),
                access_key_id: env::var("S3_ACCESS_KEY_ID")?,
                secret_access_key: env::var("S3_SECRET_ACCESS_KEY")?,
                presigned_url_expiry: Duration::from_secs(
                    env::var("S3_PRESIGNED_URL_EXPIRY_SECS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(3600),
                ),
            },
        };

        Ok(config)
    }
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        self.config.connection_string()
    }
}
