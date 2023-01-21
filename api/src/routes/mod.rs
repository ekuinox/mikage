use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router,
};
use core::{services::UserService, AppState};
use reqwest::{header::LOCATION, StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackQueryParam {
    pub code: String,
    pub state: String,
}

async fn login(State(state): State<AppState>) -> impl IntoResponse {
    // TODO: 既にログインしているか
    let url = match UserService::new(state).create_spotify_redirect_url() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("{e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new());
        }
    };
    let mut header = HeaderMap::new();
    header.append(LOCATION, url.to_string().parse().unwrap());
    (StatusCode::TEMPORARY_REDIRECT, header)
}

async fn callback(
    Query(query): Query<CallbackQueryParam>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let (spotify, user) = match UserService::new(state)
        .exchange_spotify_code(query.code, query.state)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new());
        }
    };
    let mut header = HeaderMap::new();
    header.append(LOCATION, "/".parse().unwrap());
    dbg!(spotify, user);
    (StatusCode::TEMPORARY_REDIRECT, header)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .with_state(state)
}
