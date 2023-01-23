use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_sessions::extractors::ReadableSession;
use core::{services::TwitterOAuth2Service, AppState};
use reqwest::{header::LOCATION, StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackQueryParam {
    pub code: String,
    pub state: String,
}

async fn login(State(state): State<AppState>, session: ReadableSession) -> impl IntoResponse {
    // FormRequestから勝手にやりたい
    let Some(user_id) = session.get::<i32>("user_id") else {
        let mut header = HeaderMap::new();
        header.append(LOCATION, "/".parse().unwrap());
        return (StatusCode::TEMPORARY_REDIRECT, header);
    };

    let Ok(service) = TwitterOAuth2Service::new_with_user_id(state, user_id).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new())
    };
    let url = match service.create_twitter_redirect_url() {
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
    session: ReadableSession,
) -> impl IntoResponse {
    // FormRequestから勝手にやりたい
    let Some(user_id) = session.get::<i32>("user_id") else {
        let mut header = HeaderMap::new();
        header.append(LOCATION, "/".parse().unwrap());
        return (StatusCode::TEMPORARY_REDIRECT, header);
    };
    dbg!(&query, user_id);
    let service = match TwitterOAuth2Service::new_with_user_id(state, user_id).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new())
        }
    };
    println!("create service ok");
    let _twitter = match service.exchange_spotify_code(query.code, query.state).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new());
        }
    };
    dbg!(&_twitter);
    let mut header = HeaderMap::new();
    header.append(LOCATION, "/".parse().unwrap());

    println!("twitter login ok");

    // ここでリダイレクトするからsessionにinsertしても飛んじゃうっぽい（どうしたらいい...
    // リダイレクトから戻ってきたところだからっぽい (Spotifyから飛ばされて戻ってきたとこ)
    // Laxにしておく必要があるっぽい
    (StatusCode::TEMPORARY_REDIRECT, header)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
}
