mod oauth2_credentials;

use derive_new::new;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use oauth2_credentials::{
    OAuth2Credentials, OAuth2CredentialsNotReady, OAuth2CredentialsReady, Service,
};

#[derive(new, Deserialize, Serialize, PartialEq, Debug)]
pub struct MikageConf {
    pub credentials: Vec<OAuth2Credentials>,
    pub spotify_playlist_id: String,
    pub log_file: Option<PathBuf>,
}
