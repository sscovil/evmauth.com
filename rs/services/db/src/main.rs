mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded");

    let db = postgres::create_pool(
        &config.pg.connection_string(),
        config.pg.pool.max_connections,
        config.pg.pool.min_connections,
    )
    .await?;
    tracing::info!("PostgreSQL connection established");

    #[cfg(not(feature = "dev-seeds"))]
    {
        sqlx::migrate!("./migrations").run(&db).await?;
        tracing::info!("PostgreSQL migrations completed");
    }

    #[cfg(feature = "dev-seeds")]
    {
        sqlx::migrate!("./migrations")
            .set_ignore_missing(true)
            .run(&db)
            .await?;
        tracing::info!("PostgreSQL migrations completed");

        sqlx::migrate!("./seeds")
            .set_ignore_missing(true)
            .run(&db)
            .await?;
        tracing::info!("PostgreSQL seeds completed");
    }

    Ok(())
}
