use redis::{Client, aio::ConnectionManager};
use serde::Deserialize;

/// Redis connection configuration
#[derive(Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis server hostname
    pub host: String,
    /// Redis server port
    pub port: u16,
    /// Optional authentication password
    pub password: Option<String>,
}

impl std::fmt::Debug for RedisConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("password", &self.password.as_ref().map(|_| "[redacted]"))
            .finish()
    }
}

impl RedisConfig {
    /// Build a Redis connection string from this configuration.
    ///
    /// WARNING: The returned string contains credentials and must not be logged.
    pub fn connection_string(&self) -> String {
        match &self.password {
            Some(password) => format!("redis://:{}@{}:{}", password, self.host, self.port),
            None => format!("redis://{}:{}", self.host, self.port),
        }
    }
}

/// Open a Redis client and return a reconnecting [`ConnectionManager`] that
/// automatically re-establishes the connection on failure.
pub async fn create_client(
    connection_string: &str,
) -> Result<ConnectionManager, redis::RedisError> {
    let client = Client::open(connection_string)?;
    ConnectionManager::new(client).await
}
