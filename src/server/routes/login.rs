use crate::{
    oauth2::{OAuth2Client, SpotifyOAuth2Client},
    server::state::State,
};
use actix_web::{get, web, HttpResponse, Responder};
use reqwest::header::LOCATION;
use sea_orm::{ActiveModelTrait, Set};
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
    let Ok(user) = SpotifyUserResponse::get(&access_token).await else {
        return HttpResponse::InternalServerError().finish();
    };

    dbg!(&user);

    // 既に存在する user.id かチェックしたい
    // spotify の user.id はユニークとして扱いたい
    // refresh_token, access_token を保持したい
    let r = entity::user::ActiveModel {
        name: Set(user.display_name),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    }
    .save(data.conn.as_ref())
    .await;
    let Ok(model) = r else {
        return HttpResponse::InternalServerError().finish();
    };
    dbg!(&model);

    // TODO: 新しいユーザー、または既存ユーザーとして見做せるようにセッションつけて返したい
    return HttpResponse::TemporaryRedirect()
        .append_header((LOCATION, "/"))
        .finish();
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
