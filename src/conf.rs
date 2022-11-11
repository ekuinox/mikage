use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf, str::FromStr, ops::Deref};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct OAuth2ClientCredential {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Conf {
    pub database: PathBuf,
    pub twitter_client_credential: OAuth2ClientCredential,
    pub spotify_client_credential: OAuth2ClientCredential,
}

#[derive(Clone, Debug)]
pub struct ConfFromPath(Conf);

impl FromStr for ConfFromPath {
    type Err = anyhow::Error;
    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer)?;
        let conf: Conf = toml::from_slice(&buffer)?;
        Ok(ConfFromPath(conf))
    }
}

impl From<ConfFromPath> for Conf {
    fn from(c: ConfFromPath) -> Self {
        c.0
    }
}

impl Deref for ConfFromPath {
    type Target = Conf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}