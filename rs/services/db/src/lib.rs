pub mod config;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub config: config::Config,
    pub db: PgPool,
}
