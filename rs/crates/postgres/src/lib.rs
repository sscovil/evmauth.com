use serde::Deserialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Connection pool acquire timeout
const ACQUIRE_TIMEOUT_SECS: u64 = 3;

/// PostgreSQL connection configuration
#[derive(Clone, Deserialize)]
pub struct PGConfig {
    /// Database server hostname
    pub host: String,
    /// Database server port
    pub port: u16,
    /// Authentication username
    pub user: String,
    /// Authentication password
    pub password: String,
    /// Database name
    pub database: String,
}

impl std::fmt::Debug for PGConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PGConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &"[redacted]")
            .field("database", &self.database)
            .finish()
    }
}

impl PGConfig {
    /// Build a PostgreSQL connection string from this configuration.
    ///
    /// WARNING: The returned string contains credentials and must not be logged.
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

/// Establish a PostgreSQL connection pool with the given size bounds and a
/// fixed acquire timeout of [`ACQUIRE_TIMEOUT_SECS`] seconds.
pub async fn create_pool(
    connection_string: &str,
    max_connections: u32,
    min_connections: u32,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(ACQUIRE_TIMEOUT_SECS))
        .connect(connection_string)
        .await
}
