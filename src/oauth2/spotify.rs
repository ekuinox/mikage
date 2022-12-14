use super::OAuth2Client;
use anyhow::Result;
use oauth2::basic::BasicClient;

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const SPOTIFY_REDIRECT_URL: &str = "http://localhost:10092/callback"; // TODO!! ビルド時のenvから取るとかする
const SPOTIFY_SCOPES: [&str; 2] = ["user-read-currently-playing", "playlist-modify-private"];

/// redirect_url の生成と code と access_token の引き換えをやる
/// csrf_token はメモリに保存しときゃいいと思う
#[derive(Debug)]
pub struct SpotifyOAuth2Client {
    inner: BasicClient,
}

impl OAuth2Client for SpotifyOAuth2Client {
    fn auth_url() -> &'static str {
        SPOTIFY_AUTH_URL
    }
    fn token_url() -> &'static str {
        SPOTIFY_TOKEN_URL
    }
    fn redirect_url() -> &'static str {
        SPOTIFY_REDIRECT_URL
    }
    fn scopes() -> &'static [&'static str] {
        &SPOTIFY_SCOPES
    }

    fn client<'a>(&'a self) -> &'a BasicClient {
        &self.inner
    }
}

impl SpotifyOAuth2Client {
    #[allow(unused)]
    pub fn new(client_id: &str, client_secret: &str) -> Result<Self> {
        let client = Self::build_client(client_id, client_secret)?;
        Ok(Self { inner: client })
    }
}
