use anyhow::{bail, ensure, Result};
pub use oauth2::Scope;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, StandardTokenResponse,
    TokenUrl,
};
use reqwest::Url;
use std::{collections::HashMap, ops::Deref};

const SPOTIFY_AUTHORIZE_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

pub struct SpotifyOAuth2Client {
    inner: BasicClient,
}

pub struct SpotifyOAuth2Authorizer {
    authorize_url: Url,
    csrf_state: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
}

impl SpotifyOAuth2Authorizer {
    /// create authorizer
    fn new(client: &SpotifyOAuth2Client, scopes: Vec<Scope>) -> SpotifyOAuth2Authorizer {
        let (code_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .set_pkce_challenge(code_challenge)
            .url();
        SpotifyOAuth2Authorizer {
            authorize_url,
            csrf_state,
            pkce_verifier,
        }
    }

    /// get token by redirect url and consume self
    async fn try_into_token_with_redirect_url(
        self,
        client: &SpotifyOAuth2Client,
        redirect_url: &str,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let redirect_url = Url::parse(redirect_url)?;
        let params = redirect_url.query_pairs().collect::<HashMap<_, _>>();
        let code = match params.get("code") {
            Some(code) => AuthorizationCode::new(code.to_string()),
            None => bail!("couldn't find pair which key is 'code'"),
        };
        let state = match params.get("state") {
            Some(state) => CsrfToken::new(state.to_string()),
            None => bail!("couldn't find pair which key is 'state'"),
        };
        ensure!(state.secret() == self.csrf_state.secret());
        let token = client
            .exchange_code(code)
            .set_pkce_verifier(self.pkce_verifier)
            .request_async(async_http_client)
            .await?;
        Ok(token)
    }

    pub fn authorize_url(&self) -> &str {
        self.authorize_url.as_str()
    }
}

impl Deref for SpotifyOAuth2Client {
    type Target = BasicClient;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SpotifyOAuth2Client {
    /// create client with callback url
    pub fn new_with_callback_url(
        client_id: String,
        client_secret: String,
        callback_url: String,
    ) -> Result<SpotifyOAuth2Client> {
        let SpotifyOAuth2Client { inner: client } = Self::new(client_id, client_secret)?;
        let redirect_url = RedirectUrl::new(callback_url)?;
        let client = client.set_redirect_uri(redirect_url);
        Ok(SpotifyOAuth2Client { inner: client })
    }

    /// create client with callback url
    pub fn new(client_id: String, client_secret: String) -> Result<SpotifyOAuth2Client> {
        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);
        let auth_url = AuthUrl::new(SPOTIFY_AUTHORIZE_URL.to_owned())?;
        let token_url = TokenUrl::new(SPOTIFY_TOKEN_URL.to_owned())?;
        let client = BasicClient::new(client_id, client_secret.into(), auth_url, token_url.into());
        Ok(SpotifyOAuth2Client { inner: client })
    }

    /// create authorizer instance
    pub fn authorizer(&self, scopes: Vec<Scope>) -> SpotifyOAuth2Authorizer {
        SpotifyOAuth2Authorizer::new(self, scopes)
    }

    /// request token by redirect url
    pub async fn get_token(
        &self,
        authorizer: SpotifyOAuth2Authorizer,
        redirect_url: &str,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        authorizer
            .try_into_token_with_redirect_url(self, redirect_url)
            .await
    }

    /// request new token
    pub async fn refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let refresh_token = RefreshToken::new(refresh_token);
        let token = self
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await?;
        Ok(token)
    }
}
