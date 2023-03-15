use anyhow::{bail, Result};
use chrono::Utc;
use entity::{twitter_account, user};
use reqwest::Url;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TryIntoModel};
use twitter_v2::{authorization::BearerToken, TwitterApi};

use crate::{AppState, TwitterOAuth2Client};

#[derive(Clone, Debug)]
pub struct TwitterOAuth2Service {
    user: user::Model,
    state: AppState,
}

impl TwitterOAuth2Service {
    pub fn new(user: user::Model, state: AppState) -> TwitterOAuth2Service {
        TwitterOAuth2Service { state, user }
    }

    pub async fn new_with_user_id(state: AppState, id: i32) -> Result<TwitterOAuth2Service> {
        let user = user::Entity::find_by_id(id).one(&state.connection).await?;
        let Some(user) = user else {
            bail!("User not found");
        };
        let user = user.try_into_model()?;
        Ok(TwitterOAuth2Service::new(user, state))
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.state.connection
    }

    pub fn twitter_oauth2_client(&self) -> Result<TwitterOAuth2Client> {
        TwitterOAuth2Client::new(
            self.state
                .oauth2_client_credentials
                .twitter
                .client_id
                .clone(),
            self.state
                .oauth2_client_credentials
                .twitter
                .client_secret
                .clone(),
            self.state
                .oauth2_client_credentials
                .twitter
                .redirect_uri
                .clone(),
        )
    }

    pub fn create_twitter_redirect_url(&self) -> Result<Url> {
        let client = self.twitter_oauth2_client()?;
        let (url, state, verifier) = client.create_authorize_urls();
        self.state.twitter_verifiers.insert(state, verifier)?;
        Ok(url)
    }

    pub async fn exchange_spotify_code(
        &self,
        code: String,
        state: String,
    ) -> Result<twitter_account::Model> {
        let verifier = self.state.twitter_verifiers.remove(&state)?;
        let client = self.twitter_oauth2_client()?;
        let (access_token, refresh_token) = client.exchange_code(verifier, code).await?;
        let Some(refresh_token) = refresh_token else {
            bail!("refresh_token is none");
        };

        let api = TwitterApi::new(BearerToken::new(access_token.clone()));
        let user = api.get_users_me().send().await?;
        let Some(user) = user.into_data() else {
            bail!("not include user data");
        };
        let user_id = user.id.as_u64().to_string();
        let avatar_url = user
            .profile_image_url
            .map(|url| url.to_string())
            .unwrap_or_default();

        let twitter = twitter_account::ActiveModel {
            user_id: Set(user_id),
            screen_name: Set(user.username),
            display_name: Set(user.name),
            avatar_url: Set(avatar_url),
            access_token: Set(access_token),
            refresh_token: Set(refresh_token),
            owner_user_id: Set(self.user.id),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        }
        .insert(self.connection())
        .await?
        .try_into_model()?;

        Ok(twitter)
    }
}
