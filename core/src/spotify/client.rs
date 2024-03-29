use anyhow::Result;
use derive_new::new;
use reqwest::RequestBuilder;
use serde::Deserialize;

#[derive(new, Debug)]
pub struct SpotifyClient {
    token: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct CurrentUsersProfile {
    pub display_name: String,
    pub id: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Track {
    pub name: String,
    pub uri: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct PlaylistItem {
    pub track: Track,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Playlist {
    pub items: Vec<PlaylistItem>,
}

impl SpotifyClient {
    fn get(&self, path: &str) -> RequestBuilder {
        reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/{path}"))
            .header("Authorization", format!("Bearer {}", self.token))
    }

    fn post(&self, path: &str) -> RequestBuilder {
        reqwest::Client::new()
            .post(format!("https://api.spotify.com/v1/{path}"))
            .header("Authorization", format!("Bearer {}", self.token))
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Playlist> {
        let r = self
            .get(&format!("playlists/{playlist_id}/tracks"))
            .send()
            .await?
            .json()
            .await?;
        Ok(r)
    }

    pub async fn add_tracks_to_playlist(
        &self,
        playlist_id: &str,
        track_uris: Vec<String>,
    ) -> Result<()> {
        let body = serde_json::json!({
            "uris": track_uris,
        });
        let _: serde_json::Value = self
            .post(&format!("playlists/{playlist_id}/tracks"))
            .body(body.to_string())
            .send()
            .await?
            .json()
            .await?;
        Ok(())
    }

    pub async fn get_current_users_profile(&self) -> Result<CurrentUsersProfile> {
        let r = self.get("/me").send().await?.json().await?;
        Ok(r)
    }

    pub async fn get_track(&self, track_id: &str) -> Result<Track> {
        let r = self
            .get(&format!("tracks/{track_id}"))
            .send()
            .await?
            .json()
            .await?;
        Ok(r)
    }
}

#[async_trait::async_trait]
pub trait AddTracksToPlaylist {
    async fn add_tracks_to_playlist<'a, 'b>(
        &mut self,
        playlist_id: &'a str,
        track_uris: Vec<&'b str>,
    ) -> Result<Vec<&'b str>>;

    async fn get_playlist_tracks<'a>(&self, playlist_id: &'a str) -> Result<Playlist>;
}

#[async_trait::async_trait]
impl AddTracksToPlaylist for SpotifyClient {
    async fn add_tracks_to_playlist<'a, 'b>(
        &mut self,
        playlist_id: &'a str,
        track_uris: Vec<&'b str>,
    ) -> Result<Vec<&'b str>> {
        let body = serde_json::json!({
            "uris": track_uris,
        });
        let _: serde_json::Value = self
            .post(&format!("playlists/{playlist_id}/tracks"))
            .body(body.to_string())
            .send()
            .await?
            .json()
            .await?;
        Ok(track_uris)
    }

    async fn get_playlist_tracks<'a>(&self, playlist_id: &'a str) -> Result<Playlist> {
        let r = self
            .get(&format!("playlists/{playlist_id}/tracks"))
            .send()
            .await?
            .json()
            .await?;
        Ok(r)
    }
}
