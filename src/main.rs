mod app;
mod conf;
mod db;
mod server;
mod service;
mod spotify;
mod twitter;

use anyhow::Result;
use clap::Parser;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, EntityTrait, prelude::{DateTimeUtc, DateTimeWithTimeZone, ChronoDateTimeUtc}, ActiveModelTrait, Set};
use server::ServerApp;
use serde::Deserialize;
use dotenvy::dotenv;

#[derive(Deserialize, Debug)]
struct Env {
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv()?;
    let env: Env = envy::from_iter(std::env::vars())?;
    dbg!(&env);


    let connection = Database::connect(&env.database_url).await?;
    Migrator::up(&connection, None).await?;

    let model = entity::user::ActiveModel {
        name: Set("ekuinox".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    }
    .save(&connection)
    .await?;
    dbg!(&model);

    let models = entity::user::Entity::find().all(&connection).await?;
    dbg!(&models);

    let server = ServerApp::try_parse()?;
    server.run().await?;
    Ok(())
}
