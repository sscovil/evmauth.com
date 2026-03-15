use serde::Deserialize;
use std::env;

const DEFAULT_POSTGRES_PORT: u16 = 5432;
const DEFAULT_POSTGRES_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_POSTGRES_MIN_CONNECTIONS: u32 = 0;
const DEFAULT_REDIS_PORT: u16 = 6379;
const DEFAULT_EVM_CHAIN_ID: &str = "31337";

#[derive(Clone)]
pub struct Config {
    pub pg: DatabaseConfig,
    pub redis: redis_client::RedisConfig,
    pub jwt_private_key_pem: Option<String>,
    pub jwt_public_key_pem: Option<String>,
    pub wallets_service_url: String,
    pub evm: evm::EvmConfig,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("pg", &self.pg)
            .field("redis", &self.redis)
            .field(
                "jwt_private_key_pem",
                &self.jwt_private_key_pem.as_ref().map(|_| "[redacted]"),
            )
            .field(
                "jwt_public_key_pem",
                &self.jwt_public_key_pem.as_ref().map(|_| "[redacted]"),
            )
            .field("wallets_service_url", &self.wallets_service_url)
            .field("evm", &self.evm)
            .finish()
    }
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
                        .unwrap_or(DEFAULT_POSTGRES_PORT),
                    user: env::var("POSTGRES_USER")?,
                    password: env::var("POSTGRES_PASSWORD")?,
                    database: env::var("POSTGRES_DB")?,
                },
                pool: DatabaseConnectionPoolConfig {
                    max_connections: env::var("POSTGRES_MAX_CONNECTIONS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(DEFAULT_POSTGRES_MAX_CONNECTIONS),
                    min_connections: env::var("POSTGRES_MIN_CONNECTIONS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(DEFAULT_POSTGRES_MIN_CONNECTIONS),
                },
            },
            redis: redis_client::RedisConfig {
                host: env::var("REDIS_HOST")?,
                port: env::var("REDIS_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_REDIS_PORT),
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
                    .unwrap_or_else(|_| DEFAULT_EVM_CHAIN_ID.to_string())
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
