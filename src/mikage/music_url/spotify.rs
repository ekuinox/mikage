use super::MusicUrl;
use anyhow::{bail, ensure, Result};
use itertools::Itertools;
use reqwest::Url;

const SPOTIFY_DOMAIN: &str = "open.spotify.com";

#[derive(PartialEq, Debug)]
pub enum Spotify {
    Track(String),
    Playlist(String),
    Album(String),
}

impl TryFrom<Url> for Spotify {
    type Error = anyhow::Error;
    fn try_from(value: Url) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&Url> for Spotify {
    type Error = anyhow::Error;
    fn try_from(url: &Url) -> Result<Self, Self::Error> {
        ensure!(url.domain() == Some(SPOTIFY_DOMAIN));

        let (kind, id) = match url
            .path()
            .split('/')
            .skip_while(|s| s.is_empty())
            .next_tuple()
        {
            Some(t) => t,
            None => bail!("couldn't find kind and id"),
        };

        let s = match kind {
            "track" => Spotify::Track(id.to_string()),
            "album" => Spotify::Album(id.to_string()),
            "playlist" => Spotify::Playlist(id.to_string()),
            _ => bail!("not matched"),
        };
        Ok(s)
    }
}

impl From<Spotify> for MusicUrl {
    fn from(s: Spotify) -> Self {
        MusicUrl::Spotify(s)
    }
}

#[test]
fn test_try_from_spotify() {
    const PLAYLIST_URL: &str =
        "https://open.spotify.com/playlist/2gCZudRzF57mzLh4PEBrgH?si=fd5a07f2a6864a2a";
    const ALBUM_URL: &str =
        "https://open.spotify.com/album/3luBggRjXhNlC2O3wQACop?si=jlQkTqETSeuKx16UC4cgyQ";
    const TRACK_URL: &str = "https://open.spotify.com/track/6pKAfMG926fZfD2Z4ooA2N";

    let playlist = Spotify::try_from(Url::parse(PLAYLIST_URL).unwrap());
    assert!(playlist.is_ok());
    assert_eq!(
        playlist.unwrap(),
        Spotify::Playlist("2gCZudRzF57mzLh4PEBrgH".to_string())
    );

    let album = Spotify::try_from(Url::parse(ALBUM_URL).unwrap());
    assert!(album.is_ok());
    assert_eq!(
        album.unwrap(),
        Spotify::Album("3luBggRjXhNlC2O3wQACop".to_string())
    );

    let track = Spotify::try_from(Url::parse(TRACK_URL).unwrap());
    assert!(track.is_ok());
    assert_eq!(
        track.unwrap(),
        Spotify::Track("6pKAfMG926fZfD2Z4ooA2N".to_string())
    );
}
