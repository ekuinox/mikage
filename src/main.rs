mod app;
mod conf;
mod db;
mod oauth2;
mod server;
mod service;
mod spotify;
mod twitter;

use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    prelude::{ChronoDateTimeUtc, DateTimeUtc, DateTimeWithTimeZone},
    ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set,
};
use serde::Deserialize;
use server::ServerApp;

#[derive(Deserialize, Debug)]
struct Env {
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // let _ = dotenv()?;
    // let env: Env = envy::from_iter(std::env::vars())?;
    // dbg!(&env);

    // let connection = Database::connect(&env.database_url).await?;
    // Migrator::up(&connection, None).await?;

    // let model = entity::user::ActiveModel {
    //     name: Set("ekuinox".to_string()),
    //     created_at: Set(chrono::Utc::now().into()),
    //     updated_at: Set(chrono::Utc::now().into()),
    //     ..Default::default()
    // }
    // .save(&connection)
    // .await?;
    // dbg!(&model);

    // let models = entity::user::Entity::find().all(&connection).await?;
    // dbg!(&models);

    let server = ServerApp::try_parse()?;
    server.run().await?;
    Ok(())
}
