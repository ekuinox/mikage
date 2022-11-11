use super::AsyncRunner;
use crate::conf::ConfFromPath;
use crate::{
    conf::{Conf, OAuth2ClientCredential},
    db::{Database, OAuth2ClientCredentialDatabase},
};
use anyhow::{ensure, Result};
use clap::Parser;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Url;
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc};
use warp::{Filter, Rejection, Reply};

const TWITTER_AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";
const TWITTER_TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";
const TWITTER_REDIRECT_URL: &str = "http://localhost:3030/twitter-callback";
const TWITTER_SCOPES: [&str; 10] = [
    "offline.access",
    "tweet.read",
    "users.read",
    "bookmark.read",
    "follows.read",
    "block.read",
    "like.read",
    "mute.read",
    "follows.read",
    "follows.read",
];

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const SPOTIFY_REDIRECT_URL: &str = "http://localhost:3030/spotify-callback";
const SPOTIFY_SCOPES: [&str; 2] = ["user-read-currently-playing", "playlist-modify-private"];

#[derive(Deserialize, Debug)]
pub struct CallbackQueryParam {
    pub code: String,
    pub state: String,
}

#[derive(Clone, Debug)]
pub struct State {
    conf: Arc<Conf>,
    db: Database,
}

impl State {
    fn new(conf: impl Into<Conf>) -> Result<State> {
        let conf: Conf = conf.into();
        let db = Database::from_path(&conf.database)?;
        let conf = Arc::new(conf);
        let state = State { conf, db };
        Ok(state)
    }
}

#[derive(Parser, Debug)]
pub struct InitSubcommand {
    #[clap(short = 'c', long = "conf", default_value = "conf.toml")]
    conf: ConfFromPath,
}

fn create_basic_client(
    cred: OAuth2ClientCredential,
    auth_url: String,
    token_url: String,
    redirect_url: String,
) -> Result<BasicClient> {
    let client_id = ClientId::new(cred.client_id.clone());
    let client_secret = ClientSecret::new(cred.client_secret);
    let auth_url = AuthUrl::new(auth_url)?;
    let token_url = TokenUrl::new(token_url)?;
    let redirect_url = RedirectUrl::new(redirect_url)?;
    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url);
    Ok(client)
}

fn create_basic_client_from_db(
    db: OAuth2ClientCredentialDatabase,
    auth_url: String,
    token_url: String,
    redirect_url: String,
) -> Result<BasicClient> {
    let client_id = ClientId::new(db.client_id()?);
    let client_secret = ClientSecret::new(db.client_secret()?);
    let auth_url = AuthUrl::new(auth_url)?;
    let token_url = TokenUrl::new(token_url)?;
    let redirect_url = RedirectUrl::new(redirect_url)?;
    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url);
    Ok(client)
}

fn create_authorize_urls(client: &BasicClient, scopes: &[&str]) -> (Url, String, String) {
    let scopes = scopes
        .iter()
        .map(|scope| Scope::new(scope.to_string()))
        .collect::<Vec<_>>();
    let (code_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorize_url, csrf_state) = client
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

#[async_trait::async_trait]
impl AsyncRunner for InitSubcommand {
    async fn run(self) -> Result<()> {
        let state = State::new(self.conf)?;

        let twitter_oauth_client = create_basic_client(
            state.conf.twitter_client_credential.clone(),
            TWITTER_AUTH_URL.to_owned(),
            TWITTER_TOKEN_URL.to_owned(),
            TWITTER_REDIRECT_URL.to_owned(),
        )?;
        let (tw_authorize_url, tw_csrf_token, tw_pkce_verifier) =
            create_authorize_urls(&twitter_oauth_client, &TWITTER_SCOPES);
        println!("tw_authorize_url => {tw_authorize_url}");
        let tw_cred = state.db.twitter_credential()?;
        tw_cred.set_client_id(&state.conf.twitter_client_credential.client_id)?;
        tw_cred.set_client_secret(&state.conf.twitter_client_credential.client_secret)?;
        tw_cred.set_csrf_state(&tw_csrf_token)?;
        tw_cred.set_pkce_verifier(&tw_pkce_verifier)?;

        let spotify_oauth_client = create_basic_client(
            state.conf.spotify_client_credential.clone(),
            SPOTIFY_AUTH_URL.to_owned(),
            SPOTIFY_TOKEN_URL.to_owned(),
            SPOTIFY_REDIRECT_URL.to_owned(),
        )?;
        let (sp_authorize_url, sp_csrf_token, sp_pkce_verifier) =
            create_authorize_urls(&spotify_oauth_client, &SPOTIFY_SCOPES);
        println!("sp_authorize_url => {sp_authorize_url}");
        let sp_cred = state.db.spotify_credentials()?;
        sp_cred.set_client_id(&state.conf.spotify_client_credential.client_id)?;
        sp_cred.set_client_secret(&state.conf.spotify_client_credential.client_secret)?;
        sp_cred.set_csrf_state(&sp_csrf_token)?;
        sp_cred.set_pkce_verifier(&sp_pkce_verifier)?;

        let routes = twitter_callback(state.clone()).or(spotify_callback(state.clone()));

        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

        Ok(())
    }
}

fn with_state(
    state: State,
) -> impl Filter<Extract = (State,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn twitter_callback(state: State) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("twitter-callback")
        .and(warp::get())
        .and(with_state(state))
        .and(warp::query())
        .and_then(twitter_callback_handler)
}

fn spotify_callback(state: State) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("spotify-callback")
        .and(warp::get())
        .and(with_state(state))
        .and(warp::query())
        .and_then(spotify_callback_handler)
}

async fn auth_twitter(state: State, query: CallbackQueryParam) -> Result<()> {
    let cred = state.db.twitter_credential()?;
    let csrf_state = cred.csrf_state()?;
    ensure!(csrf_state == query.state);
    let pkce_verifier = cred.pkce_verifier()?;
    let client = create_basic_client_from_db(
        cred.clone(),
        TWITTER_AUTH_URL.to_owned(),
        TWITTER_TOKEN_URL.to_owned(),
        TWITTER_REDIRECT_URL.to_owned(),
    )?;
    let token = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(async_http_client)
        .await?;

    let access_token = token.access_token().secret().as_str();
    let refresh_token = token.refresh_token().map(|t| t.secret().as_str());
    println!("twitter_access_token = {access_token}");
    println!("twitter_refresh_token = {refresh_token:?}");

    cred.set_access_token(access_token)?;
    if let Some(refresh_token) = refresh_token {
        cred.set_refresh_token(refresh_token)?
    }

    let _ = cred.drop_csrf_state()?;
    let _ = cred.drop_pkce_verifier()?;
    Ok(())
}

async fn auth_spotify(state: State, query: CallbackQueryParam) -> anyhow::Result<()> {
    let cred = state.db.spotify_credentials()?;
    let csrf_state = cred.csrf_state()?;
    ensure!(csrf_state == query.state);
    let pkce_verifier = cred.pkce_verifier()?;
    let client = create_basic_client_from_db(
        cred.clone(),
        SPOTIFY_AUTH_URL.to_owned(),
        SPOTIFY_TOKEN_URL.to_owned(),
        SPOTIFY_REDIRECT_URL.to_owned(),
    )?;
    let token = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(async_http_client)
        .await?;

    let access_token = token.access_token().secret().as_str();
    let refresh_token = token.refresh_token().map(|t| t.secret().as_str());
    println!("spotify_access_token = {access_token}");
    println!("spotify_refresh_token = {refresh_token:?}");

    cred.set_access_token(access_token)?;
    if let Some(refresh_token) = refresh_token {
        cred.set_refresh_token(refresh_token)?
    }

    let _ = cred.drop_csrf_state()?;
    let _ = cred.drop_pkce_verifier()?;
    Ok(())
}

async fn twitter_callback_handler(
    state: State,
    query: CallbackQueryParam,
) -> Result<impl Reply, Infallible> {
    let resp = match auth_twitter(state, query).await {
        Ok(_) => warp::reply::html("OK".to_string()),
        Err(e) => {
            eprintln!("{e}");
            eprint!("{}", e.backtrace());
            warp::reply::html(format!("{e}"))
        }
    };
    Ok(resp)
}

async fn spotify_callback_handler(
    state: State,
    query: CallbackQueryParam,
) -> Result<impl Reply, Infallible> {
    let resp = match auth_spotify(state, query).await {
        Ok(_) => warp::reply::html("OK".to_string()),
        Err(e) => {
            eprintln!("{e}");
            eprint!("{}", e.backtrace());
            warp::reply::html(format!("{e}"))
        }
    };
    Ok(resp)
}
