use super::{
    conf::{
        MikageConf, OAuth2Credentials, OAuth2CredentialsNotReady, OAuth2CredentialsReady, Service,
    },
    MusicUrl, Spotify,
};
use crate::{
    spotify::{Playlist, PlaylistItem, SpotifyClient, SpotifyOAuth2Client, Track},
    twitter::{TimelineReader, Tweet},
};
use anyhow::{bail, Result};
use clap::Parser;
use futures::prelude::*;
use oauth2::{Scope, TokenResponse};
use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

// 最大何回タイムラインを取得するか
const MAX_TWEET_FETCHES: usize = 10;

#[derive(Parser, Debug)]
pub struct App {
    pub conf: PathBuf,
}

impl App {
    pub async fn run(self) -> Result<()> {
        let App { conf: path, .. } = self;
        let MikageConf {
            credentials,
            spotify_playlist_id,
            log_file,
            db,
        } = read_conf(&path).await?;

        use log::LevelFilter;
        use log4rs::{
            append::{
                console::{ConsoleAppender, Target},
                file::FileAppender,
            },
            config::{Appender, Config, Root},
            encode::pattern::PatternEncoder,
            filter::threshold::ThresholdFilter,
        };

        let mut appenders = Vec::<(&str, Appender)>::new();
        appenders.push((
            "stderr",
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
                .build(
                    "stderr",
                    Box::new(ConsoleAppender::builder().target(Target::Stderr).build()),
                ),
        ));
        if let Some(path) = &log_file {
            if let Ok(file) = FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(
                    "[{d(%Y-%m-%d %H:%M:%S)} {l}]: {m}\n",
                )))
                .build(path.to_owned())
            {
                appenders.push((
                    "logfile",
                    Appender::builder().build("logfile", Box::new(file)),
                ));
            }
        }
        let (appender_names, appenders): (Vec<&str>, Vec<Appender>) = appenders.into_iter().unzip();
        let log_conf = Config::builder().appenders(appenders).build(
            Root::builder()
                .appenders(appender_names)
                .build(LevelFilter::Trace),
        )?;
        let _ = log4rs::init_config(log_conf)?;

        let credentials = {
            let mut refreshed = vec![];
            for cred in credentials {
                refreshed.push(refresh_credentials(cred).await?);
            }
            refreshed
        };

        let conf = MikageConf::new(credentials, spotify_playlist_id, log_file, db);
        write_conf(&path, &conf).await?;
        let MikageConf {
            credentials,
            spotify_playlist_id,
            db,
            ..
        } = conf;

        let db = sled::open(&db)?;

        let tokens = credentials
            .into_iter()
            .flat_map(|credentials| {
                get_access_token(&credentials)
                    .map(|token| (credentials.service().to_owned(), token))
            })
            .collect::<HashMap<_, _>>();
        let get_token = |service: Service| -> Result<&str> {
            match tokens.get(&service) {
                Some(t) => Ok(t),
                None => bail!("{service:?} credentials is none"),
            }
        };
        let spotify = get_token(Service::Spotify)
            .map(ToOwned::to_owned)
            .map(SpotifyClient::new)?;
        let mut timeline_reader = {
            let token = get_token(Service::Twitter).map(ToOwned::to_owned)?;
            TimelineReader::new(token).await?
        };

        // 前回実行した際の一番新しいツイートIDを持ってくる
        let prev_first_tweet_id = match db.get("first_tweet_id") {
            Ok(Some(bytes)) if bytes.len() == 8 => {
                let mut bytes_: [u8; 8] = [0; 8];
                bytes_.copy_from_slice(bytes.as_ref());
                u64::from_ne_bytes(bytes_).into()
            },
            _ => None,
        };

        // 一番新しいツイートIDを残しておく
        let mut first_tweet_id = None;

        log::info!("start collecting tweets");
        for _ in 0..MAX_TWEET_FETCHES {
            // このループで最後であればtrueにする
            let mut last = false;
            let tweets = match timeline_reader.next().await {
                Ok(tweets) => tweets,
                Err(_) => continue,
            };
            let tweets = if let Some(id) = prev_first_tweet_id {
                let len = tweets.len();
                let tweets = tweets.into_iter().take_while(|tweet| tweet.id != id).collect::<Vec<_>>();
                // 前回実行した際の一番新しいツイートが見つかったとして、このループを最終にする
                if len != tweets.len() {
                    last = true;
                }
                tweets
            } else {
                tweets
            };
            if first_tweet_id.is_none() {
                first_tweet_id = tweets.first().map(|tweet| tweet.id);
            }
            let urls = tweets
                .into_iter()
                .filter(|Tweet { author_id, .. }| *author_id != timeline_reader.me())
                .flat_map(|Tweet { urls, .. }| urls.into_iter().flat_map(MusicUrl::try_from));
            let tracks = urls
                .into_iter()
                .filter_map(|url| match url {
                    MusicUrl::Spotify(Spotify::Track(track_id)) => Some(track_id),
                    _ => None,
                })
                .collect::<HashSet<_>>();

            let Playlist { items, .. } = spotify.get_playlist_tracks(&spotify_playlist_id).await?;
            let uris_already_contained = items
                .into_iter()
                .map(
                    |PlaylistItem {
                         track: Track { uri, .. },
                         ..
                     }| uri,
                )
                .collect::<HashSet<_>>();
            let tracks =
                future::join_all(tracks.iter().map(|track_id| spotify.get_track(track_id)))
                    .await
                    .into_iter()
                    .flatten()
                    .map(|Track { uri, .. }| uri)
                    .filter(|uri| !uris_already_contained.contains(uri))
                    .collect::<Vec<_>>();

            log::info!("{tracks:?}");

            if tracks.is_empty() {
                log::debug!("tracks are empty");
                continue;
            }

            if let Err(e) = spotify
                .add_tracks_to_playlist(&spotify_playlist_id, tracks)
                .await
            {
                log::error!("{e}");
            } else {
                log::info!("ok");
            }

            if last {
                break;
            }
        }

        if let Some(first_tweet_id) = first_tweet_id {
            if let Err(e) = db.insert("first_tweet_id", &first_tweet_id.to_ne_bytes()) {
                eprintln!("{e}");
            }
        }

        Ok(())
    }
}

async fn write_conf(path: &Path, conf: &MikageConf) -> Result<()> {
    let json = serde_json::to_string_pretty(conf)?;
    let mut file = OpenOptions::new().truncate(true).write(true).open(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

async fn read_conf(path: &Path) -> Result<MikageConf> {
    let buffer = {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer)?;
        buffer
    };
    let conf = serde_json::from_slice(&buffer)?;
    Ok(conf)
}

fn get_access_token(credentials: &OAuth2Credentials) -> Result<String> {
    let access_token = match credentials {
        OAuth2Credentials::External(_, path) => {
            use serde::Deserialize;
            #[derive(Deserialize, Debug)]
            pub struct ExternalFileCredentials {
                pub access_token: String,
            }
            let buffer = {
                let mut buffer = Vec::new();
                let mut file = File::open(path)?;
                let _ = file.read_to_end(&mut buffer)?;
                buffer
            };
            let ExternalFileCredentials { access_token, .. } = serde_json::from_slice(&buffer)?;
            access_token
        }
        OAuth2Credentials::NotReady(_) => bail!("credentials are not ready."),
        OAuth2Credentials::Ready(OAuth2CredentialsReady { access_token, .. }) => {
            access_token.to_owned()
        }
    };
    Ok(access_token)
}

async fn refresh_credentials(credentials: OAuth2Credentials) -> Result<OAuth2Credentials> {
    let credentials = match credentials {
        OAuth2Credentials::External(service, path) => OAuth2Credentials::External(service, path),
        OAuth2Credentials::Ready(OAuth2CredentialsReady {
            service,
            client_id,
            client_secret,
            refresh_token,
            callback_url,
            ..
        }) => {
            let (access_token, refresh_token) =
                refresh(&service, &client_id, &client_secret, refresh_token).await?;
            OAuth2CredentialsReady::new(
                service,
                client_id,
                client_secret,
                callback_url,
                access_token,
                refresh_token,
            )
            .into()
        }
        OAuth2Credentials::NotReady(OAuth2CredentialsNotReady {
            service,
            client_id,
            client_secret,
            callback_url,
            ..
        }) => {
            let (access_token, refresh_token) =
                authorize(&service, &client_id, &client_secret, &callback_url).await?;
            OAuth2CredentialsReady::new(
                service,
                client_id,
                client_secret,
                callback_url,
                access_token,
                refresh_token,
            )
            .into()
        }
    };
    Ok(credentials)
}

async fn refresh(
    service: &Service,
    client_id: &str,
    client_secret: &str,
    refresh_token: String,
) -> Result<(String, String)> {
    match service {
        Service::Twitter => unimplemented!(),
        Service::Spotify => {
            let client =
                SpotifyOAuth2Client::new(client_id.to_string(), client_secret.to_string())?;
            let token = client.refresh_token(refresh_token).await?;
            let refresh_token = match token.refresh_token() {
                Some(r) => r.secret().to_owned(),
                None => bail!(""),
            };
            let access_token = token.access_token().secret().to_owned();
            Ok((access_token, refresh_token))
        }
    }
}

async fn authorize(
    service: &Service,
    client_id: &str,
    client_secret: &str,
    callback_url: &str,
) -> Result<(String, String)> {
    match service {
        Service::Twitter => unimplemented!(),
        Service::Spotify => {
            let client = SpotifyOAuth2Client::new_with_callback_url(
                client_id.to_string(),
                client_secret.to_string(),
                callback_url.to_string(),
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

            Ok((access_token, refresh_token))
        }
    }
}
