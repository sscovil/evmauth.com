use serde::Deserialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct PGConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl PGConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

/// Create a PostgreSQL connection pool
pub async fn create_pool(
    connection_string: &str,
    max_connections: u32,
    min_connections: u32,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(3))
        .connect(connection_string)
        .await
}
