use anyhow::{bail, Result};
use chrono::Utc;
use entity::{spotify_account, user};
use reqwest::Url;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TryIntoModel};
use serde::Deserialize;

use crate::{AppState, SpotifyOAuth2Client};

#[derive(Clone, Debug)]
pub struct UserService {
    state: AppState,
}

impl UserService {
    pub fn new(state: AppState) -> UserService {
        UserService { state }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.state.connection
    }

    pub fn spotify_oauth2_client(&self) -> Result<SpotifyOAuth2Client> {
        SpotifyOAuth2Client::new(
            self.state
                .oauth2_client_credentials
                .spotify
                .client_id
                .clone(),
            self.state
                .oauth2_client_credentials
                .spotify
                .client_id
                .clone(),
            self.state
                .oauth2_client_credentials
                .spotify
                .redirect_uri
                .clone(),
        )
    }

    pub fn create_spotify_redirect_url(&self) -> Result<Url> {
        let client = self.spotify_oauth2_client()?;
        let (url, state, verifier) = client.create_authorize_urls();
        let Ok(mut verifiers) = self.state.verifiers.lock() else {
            bail!("Lock failed");
        };
        verifiers.insert(state, verifier);
        Ok(url)
    }

    pub async fn exchange_spotify_code(
        &self,
        code: String,
        state: String,
    ) -> Result<(user::Model, spotify_account::Model)> {
        let verifier = {
            let Ok(mut verifiers) = self.state.verifiers.lock() else {
                bail!("Lock failed");
            };
            let Some(verifier) = verifiers.remove(&state) else {
                bail!("Verifier is none");
            };
            verifier
        };
        let client = self.spotify_oauth2_client()?;
        let (access_token, refresh_token) = client.exchange_code(verifier, code).await?;
        let Some(refresh_token) = refresh_token else {
            bail!("refresh_token is none");
        };

        #[derive(Deserialize, Debug)]
        struct Image {
            url: String,
        }

        #[derive(Deserialize, Debug)]
        struct SpotifyUserResponse {
            display_name: String,
            id: String,
            images: Vec<Image>,
        }

        let SpotifyUserResponse {
            id: user_id,
            display_name,
            images,
        } = reqwest::Client::builder()
            .build()?
            .get("https://api.spotify.com/v1/me")
            .bearer_auth(&access_token)
            .send()
            .await?
            .json::<SpotifyUserResponse>()
            .await?;
        let avatar_url = images
            .into_iter()
            .map(|img| img.url)
            .nth(0)
            .unwrap_or_default();

        if let Ok(Some((mut spotify_account, Some(user)))) =
            spotify_account::Entity::find_by_id(user_id.clone())
                .find_also_related(user::Entity)
                .one(self.connection())
                .await
        {
            spotify_account.display_name = display_name;
            spotify_account.access_token = access_token;
            spotify_account.refresh_token = refresh_token;
            spotify_account.updated_at = Utc::now().into();
            if !avatar_url.is_empty() {
                spotify_account.avatar_url = avatar_url;
            }
            let spotify_account: spotify_account::ActiveModel = spotify_account.into();
            let spotify_account = spotify_account.update(self.connection()).await?;
            return Ok((user, spotify_account));
        }

        let user = user::ActiveModel {
            name: Set(display_name.clone()),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        }
        .insert(self.connection())
        .await?
        .try_into_model()?;

        let spotify = spotify_account::ActiveModel {
            user_id: Set(user_id),
            display_name: Set(display_name),
            avatar_url: Set(avatar_url),
            access_token: Set(access_token),
            refresh_token: Set(refresh_token),
            owner_user_id: Set(user.id),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        }
        .insert(self.connection())
        .await?
        .try_into_model()?;

        Ok((user, spotify))
    }
}
