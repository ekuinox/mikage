mod mikage;
mod spotify;
mod twitter;

use anyhow::Result;
use clap::Parser;
use mikage::App;

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::try_parse()?;
    app.run().await
}
