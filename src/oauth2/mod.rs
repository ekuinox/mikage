mod spotify;

use anyhow::Result;
use oauth2::{basic::BasicClient, AuthorizationCode, PkceCodeVerifier, TokenResponse, reqwest::async_http_client};
use reqwest::Url;

pub use self::spotify::SpotifyOAuth2Client;

#[async_trait::async_trait]
pub trait OAuth2Client {
    fn scopes() -> &'static [&'static str];
    fn token_url() -> &'static str;
    fn auth_url() -> &'static str;
    fn redirect_url() -> &'static str;
    fn client<'a>(&'a self) -> &'a BasicClient;
    fn build_client(client_id: &str, client_secret: &str) -> Result<BasicClient> {
        use oauth2::*;
        let client_id = ClientId::new(client_id.to_string().to_string());
        let client_secret = ClientSecret::new(client_secret.to_string().to_string());
        let auth_url = AuthUrl::new(Self::auth_url().to_string())?;
        let token_url = TokenUrl::new(Self::token_url().to_string())?;
        let redirect_url = RedirectUrl::new(Self::redirect_url().to_string())?;
        let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
            .set_redirect_uri(redirect_url);
        Ok(client)
    }
    fn create_authorize_urls(&self) -> (Url, String, String) {
        use oauth2::*;
        let scopes = Self::scopes()
            .iter()
            .map(|scope| Scope::new(scope.to_string()))
            .collect::<Vec<_>>();
        let (code_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = self
            .client()
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
    async fn exchange_code(
        &self,
        verifier: String,
        code: String,
    ) -> Result<(String, Option<String>)> {
        let token = self
            .client()
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(async_http_client)
            .await?;
        let access_token = token.access_token().secret().to_owned();
        let refresh_token = token.refresh_token().map(|s| s.secret().to_owned());
        Ok((access_token, refresh_token))
    }
}
