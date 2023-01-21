use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
pub struct AppState {
    pub connection: DatabaseConnection,
    pub verifiers: Arc<Mutex<HashMap<String, String>>>,
    pub oauth2_client_credentials: Arc<OAuth2ClientCredentials>,
}

impl AppState {
    pub fn new(
        connection: DatabaseConnection,
        oauth2_client_credentials: OAuth2ClientCredentials,
    ) -> AppState {
        AppState {
            connection,
            verifiers: Arc::new(Mutex::new(Default::default())),
            oauth2_client_credentials: Arc::new(oauth2_client_credentials),
        }
    }
}
