mod mikage;
mod spotify;
mod twitter;

use anyhow::{bail, Result};
use clap::Parser;
use mikage::MusicUrl;
use oauth2::TokenResponse;
use serde::{Deserialize, Serialize};
use spotify::{Scope, SpotifyOAuth2Client};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};
use twitter::{TimelineReader, Tweet};

#[derive(Parser, Debug)]
pub struct App {
    pub configuration_path: PathBuf,

    #[clap(short = 't', long)]
    pub twitter_credentials_path: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Configuration {
    pub spotify_credentials: SpotifyCredentials,
}

#[derive(Deserialize, Debug)]
pub struct TwitterCredentials {
    access_token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyReadyCredentials {
    client_id: String,
    client_secret: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum SpotifyCredentials {
    Init {
        client_id: String,
        client_secret: String,
        callback_url: String,
    },
    Ready(SpotifyReadyCredentials),
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::try_parse()?;

    let Configuration {
        spotify_credentials,
        ..
    } = serde_json::from_reader(File::open(&app.configuration_path)?)?;
    let TwitterCredentials {
        access_token: twitter_token,
        ..
    } = serde_json::from_reader(File::open(&app.twitter_credentials_path)?)?;

    let (_, cred) = match spotify_credentials {
        SpotifyCredentials::Init {
            client_id,
            client_secret,
            callback_url,
        } => {
            let client = SpotifyOAuth2Client::new_with_callback_url(
                client_id.clone(),
                client_secret.clone(),
                callback_url,
            )?;
            let scopes = ["user-read-currently-playing", "playlist-modify-private"]
                .into_iter()
                .map(|scope| Scope::new(scope.to_owned()))
                .collect::<Vec<_>>();
            let authorizer = client.authorizer(scopes);
            let authorize_url = authorizer.authorize_url();
            println!("{authorize_url}");
            let buffer = {
                let mut buffer = String::new();
                let _ = std::io::stdin().read_line(&mut buffer)?;
                buffer
            };
            let token = client.get_token(authorizer, &buffer).await?;
            let refresh_token = match token.refresh_token() {
                Some(r) => r.secret().to_owned(),
                None => bail!(""),
            };
            let access_token = token.access_token().secret().to_owned();
            let cred = SpotifyReadyCredentials {
                client_id,
                client_secret,
                access_token: access_token.clone(),
                refresh_token,
            };
            (client, cred)
        }
        SpotifyCredentials::Ready(SpotifyReadyCredentials {
            client_id,
            client_secret,
            refresh_token,
            ..
        }) => {
            let client = SpotifyOAuth2Client::new(client_id.clone(), client_secret.clone())?;
            let token = client.refresh_token(refresh_token).await?;
            let refresh_token = match token.refresh_token() {
                Some(r) => r.secret().to_owned(),
                None => bail!(""),
            };
            let access_token = token.access_token().secret().to_owned();
            let cred = SpotifyReadyCredentials {
                client_id,
                client_secret,
                access_token: access_token.clone(),
                refresh_token,
            };
            (client, cred)
        }
    };

    let spotify_token = cred.access_token.clone();
    let mut file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(&app.configuration_path)?;
    let conf = Configuration {
        spotify_credentials: SpotifyCredentials::Ready(cred),
    };
    let json = serde_json::to_string(&conf)?;
    let _ = file.write(json.as_bytes())?;

    dbg!(&spotify_token);

    let mut reader = TimelineReader::new(twitter_token).await?;
    for _ in 0..1 {
        use mikage::Spotify;

        let tweets = reader.next().await?;
        let urls: Vec<MusicUrl> = tweets
            .into_iter()
            .flat_map(|Tweet { urls, .. }| urls.into_iter().flat_map(MusicUrl::try_from))
            .collect();
        let tracks = urls
            .into_iter()
            .filter_map(|url| match url {
                MusicUrl::Spotify(Spotify::Track(track_id)) => Some(track_id),
                _ => None,
            })
            .collect::<Vec<_>>();
        for track in tracks {
            println!("{track}");
            let r: serde_json::Value = reqwest::Client::new()
                .get(format!("https://api.spotify.com/v1/tracks/{track}"))
                .header("Authorization", format!("Bearer {spotify_token}"))
                .send()
                .await?
                .json()
                .await?;
            dbg!(r);
        }
    }

    Ok(())
}
