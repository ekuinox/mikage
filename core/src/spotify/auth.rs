use std::ops::Deref;

use anyhow::Result;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, url::Url, AuthorizationCode, PkceCodeVerifier,
    TokenResponse,
};

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
/// https://developer.spotify.com/documentation/general/guides/authorization/scopes/
/// https://developer.spotify.com/console/
const SPOTIFY_SCOPES: [&str; 5] = [
    "user-read-currently-playing",
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-private",
    "playlist-modify-public",
];

#[derive(Debug)]
pub struct SpotifyOAuth2Client {
    inner: BasicClient,
}

impl Deref for SpotifyOAuth2Client {
    type Target = BasicClient;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SpotifyOAuth2Client {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Result<SpotifyOAuth2Client> {
        use oauth2::*;
        let client_id = ClientId::new(client_id.to_string());
        let client_secret = ClientSecret::new(client_secret.to_string());
        let auth_url = AuthUrl::new(SPOTIFY_AUTH_URL.to_string())?;
        let token_url = TokenUrl::new(SPOTIFY_TOKEN_URL.to_string())?;
        let redirect_url = RedirectUrl::new(redirect_url)?;
        let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
            .set_redirect_uri(redirect_url);
        Ok(SpotifyOAuth2Client { inner: client })
    }

    pub fn create_authorize_urls(&self) -> (Url, String, String) {
        use oauth2::*;
        let scopes = SPOTIFY_SCOPES
            .iter()
            .map(|scope| Scope::new(scope.to_string()))
            .collect::<Vec<_>>();
        let (code_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = self
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .set_pkce_challenge(code_challenge)
            .url();
        (
            authorize_url,
            csrf_state.secret().to_string(),
            pkce_verifier.secret().to_string(),
        )
    }


    pub async fn a(&self) -> Result<(String, String)> {
        Ok(("".to_string(), "".to_string()))
    }

    pub async fn exchange_code(
        &self,
        verifier: String,
        code: String,
    ) -> Result<(String, Option<String>)> {
        let token = self
            .inner
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(async_http_client)
            .await?;
        let access_token = token.access_token().secret().to_owned();
        let refresh_token = token.refresh_token().map(|s| s.secret().to_owned());
        Ok((access_token, refresh_token))
    }
}
