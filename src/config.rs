use std::{
    fs::File,
    io::{BufReader, Read},
    net::SocketAddr,
    path::Path,
};

use anyhow::Result;
use core::OAuth2ClientCredentials;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct MikageConfig {
    pub credentials: OAuth2ClientCredentials,
    pub addr: SocketAddr,
    pub db: String,
    pub secret: String,
}

impl MikageConfig {
    pub fn open(path: &Path) -> Result<MikageConfig> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::with_capacity(512);
        reader.read_to_end(&mut buffer)?;
        let config = toml::from_slice(&buffer)?;
        Ok(config)
    }
}
