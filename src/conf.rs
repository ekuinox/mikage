use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
