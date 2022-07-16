mod mikage;
mod twitter;

use anyhow::Result;
use clap::Parser;
use mikage::MusicUrl;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::PathBuf};
use twitter::{TimelineReader, Tweet};

#[derive(Parser, Debug)]
pub struct App {
    pub path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct TwitterCredentials {
    access_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::try_parse()?;
    let f = File::open(&app.path)?;
    let reader = BufReader::new(f);
    let TwitterCredentials { access_token, .. } = serde_json::from_reader(reader)?;

    let mut reader = TimelineReader::new(access_token).await?;
    for _ in 0..1 {
        let tweets = reader.next().await?;
        let urls: Vec<MusicUrl> = tweets
            .into_iter()
            .flat_map(|Tweet { urls, .. }| urls.into_iter().flat_map(MusicUrl::try_from))
            .collect();
        dbg!(&urls);
    }

    Ok(())
}
