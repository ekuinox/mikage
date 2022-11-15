use std::collections::HashSet;

use anyhow::{bail, ensure, Result};
use itertools::Itertools;
use reqwest::Url;

use crate::{
    spotify::{AddTracksToPlaylist, Playlist, PlaylistItem},
    twitter::{GetTimeline, Tweet},
};

#[async_trait::async_trait]
pub trait AddMusicsFromTimeline {
    async fn add_musics_from_timeline(&mut self, playlist_id: &str) -> Result<Vec<String>>;
}

#[derive(PartialEq, Eq, Debug)]
pub struct PlaylistCreator<T, P>
where
    T: GetTimeline + Send + Sync,
    P: AddTracksToPlaylist + Send + Sync,
{
    timeline: T,
    playlist: P,
}

impl<T, P> PlaylistCreator<T, P>
where
    T: GetTimeline + Send + Sync,
    P: AddTracksToPlaylist + Send + Sync,
{
    pub fn new(timeline: T, playlist: P) -> PlaylistCreator<T, P> {
        PlaylistCreator { timeline, playlist }
    }
}

#[async_trait::async_trait]
impl<T, P> AddMusicsFromTimeline for PlaylistCreator<T, P>
where
    T: GetTimeline + Send + Sync,
    P: AddTracksToPlaylist + Send + Sync,
{
    async fn add_musics_from_timeline(&mut self, playlist_id: &str) -> Result<Vec<String>> {
        // プレイリストに登録済みのトラックを取得して
        let Playlist { items } = self.playlist.get_playlist_tracks(playlist_id).await?;
        let existed_tracks = items
            .into_iter()
            .map(|PlaylistItem { track }| track.uri)
            .collect::<HashSet<_>>();
        // タイムラインからSpotifyのトラックを探して
        let tweets = self.timeline.get_timeline().await?;
        let tweet_and_uris = tweets
            .iter()
            .flat_map(|tweet| get_spotify_track_uris(&tweet).map(|uris| (tweet, uris)))
            .collect::<Vec<_>>();
        // 追加すべきトラックのリストを作って
        let track_uris = tweet_and_uris
            .iter()
            .map(|(_, uris)| uris)
            .flatten()
            .filter(|uri| !existed_tracks.contains(**uri))
            .map(|uri| *uri)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        // プレイリストに追加する
        let tracks = self
            .playlist
            .add_tracks_to_playlist(playlist_id, track_uris)
            .await?
            .into_iter()
            .map(|uri| uri.to_string())
            .collect::<Vec<_>>();
        Ok(tracks)
    }
}

const SPOTIFY_DOMAIN: &str = "open.spotify.com";

fn get_spotify_track_uri(url: &Url) -> Result<&str> {
    ensure!(url.domain() == Some(SPOTIFY_DOMAIN));
    let Some(mut segments) = url.path_segments() else {
        bail!("segments not found");
    };
    let Some((kind, id)) = segments.next_tuple() else {
        bail!("not found kind and id");
    };
    ensure!(kind == "track");
    Ok(id)
}

fn get_spotify_track_uris(tweet: &Tweet) -> Option<Vec<&str>> {
    let urls = tweet
        .urls
        .iter()
        .flat_map(get_spotify_track_uri)
        .collect::<Vec<_>>();
    if urls.is_empty() {
        None
    } else {
        Some(urls)
    }
}
