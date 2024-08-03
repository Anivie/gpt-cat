use sqlx::Pool;
use sqlx_postgres::{PgPoolOptions, Postgres};
use crate::data::config::config_file::Config;

/// Connect to the database, I am use PostgreSQL in the debug environment.
/// # Arguments
/// - config: The config of the server.
pub async fn connect_to_database_sqlx(config: &Config) -> anyhow::Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .connect(config.database_url.as_str()).await?;

    Ok(pool)
}