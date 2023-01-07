mod mutation;
mod query;

pub mod spotify;
pub mod twitter;

pub use self::{
    mutation::Mutation, query::Query, spotify::SpotifyOAuth2Client, twitter::TwitterOAuth2Client,
};
