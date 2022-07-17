mod client;
mod oauth2_client;

pub use self::{
    client::{SpotifyClient, TrackResponse},
    oauth2_client::{Scope, SpotifyOAuth2Authorizer, SpotifyOAuth2Client},
};
