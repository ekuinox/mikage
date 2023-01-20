mod config;

use std::path::Path;

use anyhow::{Result, bail};
use api::{AppState, serve};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

use self::config::MikageConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        bail!("パスが指定されてないぽよ～");
    };
    let path = Path::new(&path);
    let config = MikageConfig::open(path)?;

    let connection = Database::connect(config.db).await?;

    Migrator::up(&connection, None).await?;

    let state = AppState::new(connection, config.credentials);
    serve(&config.addr, state).await?;

    Ok(())
}
