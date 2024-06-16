use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::data::config::config_file::Config;

/// Connect to the database, I am use PostgreSQL in the debug environment.
/// # Arguments
/// - config: The config of the server.
pub async fn connect_to_database(config: &Config) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(config.database_url.clone());
    opt.sqlx_logging(false);
    let db = Database::connect(opt).await?;

    Ok(db)
}