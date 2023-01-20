use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sea_orm::DatabaseConnection;

#[derive(PartialEq, Eq, Debug)]
pub struct OAuth2ClientCredential {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(PartialEq, Eq, Debug)]
pub struct OAuth2ClientCredentials {
    pub twitter: OAuth2ClientCredential,
    pub spotify: OAuth2ClientCredential,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub connections: Arc<DatabaseConnection>,
    pub verifiers: Arc<Mutex<HashMap<String, String>>>,
    pub oauth2_client_credentials: Arc<OAuth2ClientCredentials>,
}
