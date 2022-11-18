mod app;
mod conf;
mod db;
mod routes;
mod service;
mod spotify;
mod twitter;

use anyhow::Result;
use app::{App, AsyncRunner};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::try_parse()?;
    app.run().await
}
