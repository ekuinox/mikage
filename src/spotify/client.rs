use anyhow::Result;
use derive_new::new;
use serde::Deserialize;

#[derive(new, Debug)]
pub struct SpotifyClient {
    token: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TrackResponse {
    name: String,
}

impl SpotifyClient {
    #[allow(unused)]
    pub async fn get_track(&self, track_id: &str) -> Result<TrackResponse> {
        let response = reqwest::Client::new()
            .get(format!("https://api.spotify.com/v1/tracks/{track_id}"))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}
