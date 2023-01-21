use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_sessions::{
    async_session::SessionStore,
    extractors::{ReadableSession, WritableSession},
    SessionLayer,
};
use core::{services::UserService, AppState};
use reqwest::{header::LOCATION, StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackQueryParam {
    pub code: String,
    pub state: String,
}

async fn index(session: ReadableSession) -> impl IntoResponse {
    if let Some(user_id) = session.get::<i32>("user_id") {
        return format!("Logged in as {user_id}");
    }
    format!("Not logged in")
}

async fn login(State(state): State<AppState>, session: ReadableSession) -> impl IntoResponse {
    if let Some(user_id) = session.get::<i32>("user_id") {
        println!("already login as {user_id}");
        let mut header = HeaderMap::new();
        header.append(LOCATION, "/".parse().unwrap());
        return (StatusCode::TEMPORARY_REDIRECT, header);
    }
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
    mut session: WritableSession,
) -> impl IntoResponse {
    let (user, _spotify) = match UserService::new(state)
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

    if let Err(e) = session.insert("user_id", user.id) {
        eprintln!("{e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new());
    }
    println!("login ok {}", user.id);

    // ここでリダイレクトするからsessionにinsertしても飛んじゃうっぽい（どうしたらいい...
    (StatusCode::TEMPORARY_REDIRECT, header)
}

pub fn router(state: AppState, session_layer: SessionLayer<impl SessionStore>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login))
        .route("/callback", get(callback))
        .layer(session_layer)
        .with_state(state)
}
