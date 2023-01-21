use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use core::{Mutation, SpotifyOAuth2Client, AppState};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackQueryParam {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize, Debug)]
struct SpotifyUserResponse {
    display_name: String,
    id: String,
}

impl SpotifyUserResponse {
    pub async fn get(token: &str) -> anyhow::Result<Self> {
        let resp = reqwest::Client::builder()
            .build()?
            .get("https://api.spotify.com/v1/me")
            .bearer_auth(token)
            .send()
            .await?
            .json::<SpotifyUserResponse>()
            .await?;
        Ok(resp)
    }
}

async fn login(State(state): State<AppState>) -> impl IntoResponse {
    let client = SpotifyOAuth2Client::new(
        state.oauth2_client_credentials.spotify.client_id.clone(),
        state.oauth2_client_credentials.spotify.client_id.clone(),
        state.oauth2_client_credentials.spotify.redirect_uri.clone(),
    );
    let Ok(client) = client else { return Redirect::to("/")};
    let Ok(mut verifiers) = state.verifiers.lock() else { return Redirect::to("/")};
    let (url, state, verifier) = client.create_authorize_urls();
    let _ = verifiers.insert(state, verifier);
    Redirect::temporary(url.as_str())
}

async fn callback(
    Query(query): Query<CallbackQueryParam>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = {
        let verifier = {
            let Ok(mut verifiers) = state.verifiers.lock() else {
                return Redirect::to("/");
            };
            let Some(verifier) = verifiers.remove(&query.state) else {
                return Redirect::to("/");
            };
            verifier
        };
        let client = SpotifyOAuth2Client::new(
            state.oauth2_client_credentials.spotify.client_id.clone(),
            state
                .oauth2_client_credentials
                .spotify
                .client_secret
                .clone(),
            state.oauth2_client_credentials.spotify.redirect_uri.clone(),
        );
        let Ok(client) = client else {
            return Redirect::to("/");
        };
        let Ok((access_token, _refresh_token)) = client.exchange_code(verifier, query.code.clone()).await else {
            return Redirect::to("/");
        };
        let Ok(user) = SpotifyUserResponse::get(&access_token).await else {
            return Redirect::to("/");
        };

        dbg!(&user);
        user
    };

    // 既に存在する user.id かチェックしたい
    // spotify の user.id はユニークとして扱いたい
    // refresh_token, access_token を保持したい
    let _r = Mutation::new(state.connection.clone())
        .create_user(user.display_name)
        .await;

    Redirect::to("/")
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .with_state(state)
}
