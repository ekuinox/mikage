use std::path::PathBuf;

use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(new, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Twitter,
    Spotify,
}

#[derive(new, Deserialize, Serialize, PartialEq, Debug)]
pub struct OAuth2CredentialsNotReady {
    pub service: Service,
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

#[derive(new, Deserialize, Serialize, PartialEq, Debug)]
pub struct OAuth2CredentialsReady {
    pub service: Service,
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(new, Deserialize, Serialize, PartialEq, Debug)]
#[serde(untagged)] // https://serde.rs/enum-representations.html#untagged
pub enum OAuth2Credentials {
    Ready(OAuth2CredentialsReady),
    NotReady(OAuth2CredentialsNotReady),
    External(Service, PathBuf),
}

impl OAuth2Credentials {
    pub fn service(&self) -> &Service {
        match self {
            Self::External(service, _) => service,
            Self::NotReady(OAuth2CredentialsNotReady { service, .. }) => service,
            Self::Ready(OAuth2CredentialsReady { service, .. }) => service,
        }
    }
}

impl From<OAuth2CredentialsNotReady> for OAuth2Credentials {
    fn from(c: OAuth2CredentialsNotReady) -> Self {
        Self::NotReady(c)
    }
}

impl From<OAuth2CredentialsReady> for OAuth2Credentials {
    fn from(c: OAuth2CredentialsReady) -> Self {
        Self::Ready(c)
    }
}
