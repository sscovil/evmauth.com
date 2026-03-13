use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub pg: DatabaseConfig,
    pub redis: redis_client::RedisConfig,
    pub jwt_private_key_pem: Option<String>,
    pub jwt_public_key_pem: Option<String>,
    pub wallets_service_url: String,
    pub evm: evm::EvmConfig,
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
            jwt_private_key_pem: env::var("JWT_PRIVATE_KEY_PEM").ok(),
            jwt_public_key_pem: env::var("JWT_PUBLIC_KEY_PEM").ok(),
            wallets_service_url: env::var("WALLETS_SERVICE_URL")
                .unwrap_or_else(|_| "http://int-wallets:8000".to_string()),
            evm: evm::EvmConfig {
                rpc_url: env::var("EVM_RPC_URL")
                    .unwrap_or_else(|_| "http://localhost:8545".to_string()),
                platform_contract_address: env::var("EVM_PLATFORM_CONTRACT_ADDRESS")?
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid EVM_PLATFORM_CONTRACT_ADDRESS: {e}"))?,
                chain_id: env::var("EVM_CHAIN_ID")
                    .unwrap_or_else(|_| "31337".to_string())
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid EVM_CHAIN_ID: {e}"))?,
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
