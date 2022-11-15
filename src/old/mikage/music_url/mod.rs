mod spotify;

use anyhow::bail;
use reqwest::Url;

pub use self::spotify::Spotify;

#[derive(PartialEq, Eq, Debug)]
pub enum MusicUrl {
    Spotify(Spotify),
}

impl TryFrom<Url> for MusicUrl {
    type Error = anyhow::Error;
    fn try_from(url: Url) -> Result<Self, Self::Error> {
        if let Ok(spotify) = Spotify::try_from(&url)?.try_into() {
            return Ok(spotify);
        }
        bail!("not matched")
    }
}
