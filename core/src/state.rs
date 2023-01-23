use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Result};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct OAuth2ClientCredential {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct OAuth2ClientCredentials {
    pub twitter: OAuth2ClientCredential,
    pub spotify: OAuth2ClientCredential,
}

#[derive(Clone, Debug)]
pub struct OAuthVerifiers(Arc<Mutex<HashMap<String, String>>>);

impl OAuthVerifiers {
    pub fn new() -> OAuthVerifiers {
        OAuthVerifiers(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn remove(&self, state: &str) -> Result<String> {
        let Ok(mut verifiers) = self.0.lock() else {
            bail!("Lock failed");
        };
        let Some(verifier) = verifiers.remove(state) else {
            bail!("Verifier is none");
        };
        Ok(verifier)
    }

    pub fn insert(&self, state: String, verifier: String) -> Result<()> {
        let Ok(mut verifiers) = self.0.lock() else {
            bail!("Lock failed");
        };
        verifiers.insert(state, verifier);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub connection: DatabaseConnection,
    pub spotify_verifiers: OAuthVerifiers,
    pub oauth2_client_credentials: Arc<OAuth2ClientCredentials>,
}

impl AppState {
    pub fn new(
        connection: DatabaseConnection,
        oauth2_client_credentials: OAuth2ClientCredentials,
    ) -> AppState {
        AppState {
            connection,
            spotify_verifiers: OAuthVerifiers::new(),
            oauth2_client_credentials: Arc::new(oauth2_client_credentials),
        }
    }
}
