mod app;
mod conf;
mod db;
mod server;
mod service;
mod spotify;
mod twitter;

use anyhow::Result;
use clap::Parser;
use server::ServerApp;

#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerApp::try_parse()?;
    server.run().await?;
    Ok(())
}
