mod config;

use std::path::Path;

use anyhow::Result;
use api::serve;
use base64::prelude::*;
use core::AppState;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

use self::config::MikageConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "mikage.toml".to_string());
    let path = Path::new(&path);
    let config = MikageConfig::open(path)?;

    let secret = BASE64_STANDARD.decode(&config.secret)?;

    let connection = Database::connect(config.db).await?;

    Migrator::up(&connection, None).await?;

    let state = AppState::new(connection, config.credentials);
    serve(&config.addr, state, &secret).await?;

    Ok(())
}
