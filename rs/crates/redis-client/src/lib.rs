use redis::{Client, aio::ConnectionManager};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

impl RedisConfig {
    pub fn connection_string(&self) -> String {
        match &self.password {
            Some(password) => format!("redis://:{}@{}:{}", password, self.host, self.port),
            None => format!("redis://{}:{}", self.host, self.port),
        }
    }
}

/// Create a Redis connection manager
pub async fn create_client(
    connection_string: &str,
) -> Result<ConnectionManager, redis::RedisError> {
    let client = Client::open(connection_string)?;
    ConnectionManager::new(client).await
}
