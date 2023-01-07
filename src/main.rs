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
    std::env::set_var("RUST_LOG", "info");
    let server = ServerApp::try_parse()?;
    server.run().await?;
    Ok(())
}
