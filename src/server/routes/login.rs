use crate::{
    oauth2::{OAuth2Client, SpotifyOAuth2Client},
    server::state::State,
};
use actix_web::{get, web, HttpResponse, Responder};
use reqwest::header::LOCATION;
use serde::Deserialize;

/// GET /login に来たものを Spotify の OAuth に飛ばす
/// この段階だと、ユーザーは作らない -> コールバックで飛んできてから作成する
#[get("/login")]
pub async fn login(data: web::Data<State>) -> impl Responder {
    let client = SpotifyOAuth2Client::new(
        &data.conf.spotify_client_credential.client_id,
        &data.conf.spotify_client_credential.client_secret,
    );
    let Ok(client) = client else {
        return HttpResponse::InternalServerError().finish();
    };
    let Ok(mut verifiers) = data.verifiers.lock() else {
        return HttpResponse::InternalServerError().finish();
    };
    let (url, state, verifier) = client.create_authorize_urls();
    let _ = verifiers.insert(state, verifier);
    HttpResponse::TemporaryRedirect()
        .append_header((LOCATION, url.to_string()))
        .finish()
}

#[derive(Debug, Deserialize)]

pub struct CallbackQueryParam {
    pub code: String,
    pub state: String,
}

#[get("/callback")]
pub async fn callback(
    data: web::Data<State>,
    query: web::Query<CallbackQueryParam>,
) -> impl Responder {
    let Ok(mut verifiers) = data.verifiers.lock() else {
        return HttpResponse::InternalServerError().finish();
    };
    let Some(verifier) = verifiers.remove(&query.state) else {
        return HttpResponse::BadRequest().finish();
    };
    let client = SpotifyOAuth2Client::new(
        &data.conf.spotify_client_credential.client_id,
        &data.conf.spotify_client_credential.client_secret,
    );
    let Ok(client) = client else {
        return HttpResponse::InternalServerError().finish();
    };
    let Ok((access_token, refresh_token)) = client.exchange_code(verifier, query.code.clone()).await else {
        return HttpResponse::InternalServerError().finish();
    };
    dbg!(access_token, refresh_token);
    // TODO: ...

    return HttpResponse::InternalServerError().finish();
}
