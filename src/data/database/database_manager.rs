use sqlx::Pool;
use sqlx_postgres::{PgPoolOptions, Postgres};

use crate::data::config::entity::config_file::Config;

/// Connect to the database, I am use PostgreSQL in the debug environment.
/// # Arguments
/// - config: The config of the server.
pub async fn connect_to_database_sqlx(config: &Config) -> anyhow::Result<Pool<Postgres>> {
    let string = &config.database_url;

    if string.is_empty() {
        return Err(anyhow::anyhow!("The database url must be set."));
    }

    let pool = PgPoolOptions::new()
        .connect(string.as_str()).await?;

    Ok(pool)
}