use anyhow::{bail, Result};
use derive_new::new;
use reqwest::Url;
use twitter_v2::{
    authorization::BearerToken,
    data::{FullTextEntities, UrlEntity},
    id::NumericId,
    meta::TweetsMeta,
    query::{TweetExpansion, TweetField, UserField},
    TwitterApi, TwitterApiWithUserCtx, User,
};

pub struct TimelineReader {
    client: TwitterApiWithUserCtx<BearerToken>,
    next_token: Option<String>,
}

#[derive(new, Debug)]
pub struct Tweet {
    pub text: String,
    pub urls: Vec<Url>,
    pub username: String,
    pub author_id: u64,
}

impl TimelineReader {
    pub async fn new(access_token: String) -> Result<TimelineReader> {
        let auth = BearerToken::new(access_token);
        let client = TwitterApi::new(auth);
        let client = client.with_user_ctx().await?;
        Ok(TimelineReader {
            client,
            next_token: None,
        })
    }

    pub fn me(&self) -> u64 {
        self.client.user_id().as_u64()
    }

    pub async fn next(&mut self) -> Result<Vec<Tweet>> {
        let req = {
            let mut req = self.client.get_my_reverse_chronological_timelines();
            req.user_fields([UserField::CreatedAt, UserField::Username, UserField::Name])
                .tweet_fields([
                    TweetField::CreatedAt,
                    TweetField::Attachments,
                    TweetField::Entities,
                ])
                .expansions([TweetExpansion::AuthorId]);
            if let Some(t) = &self.next_token {
                req.pagination_token(t);
            }
            req
        };
        let res = req.send().await?;

        if let Some(next_token) = res
            .meta()
            .and_then(|TweetsMeta { next_token, .. }| next_token.as_ref())
        {
            self.next_token = next_token.to_owned().into();
        }

        let users = res.includes().and_then(|includes| includes.users.as_ref());
        let get_user = move |user_id: NumericId| -> Option<&User> {
            users.and_then(|users| users.iter().find(|user| user.id == user_id))
        };
        let tweets = match res.data() {
            Some(tweets) => tweets,
            None => bail!("no tweets"),
        };
        let tweets = tweets
            .iter()
            .flat_map(
                |twitter_v2::Tweet {
                     text,
                     author_id,
                     entities,
                     ..
                 }| {
                    fn to_expanded_urls(urls: &[UrlEntity]) -> Vec<Url> {
                        urls.iter()
                            .flat_map(|UrlEntity { expanded_url, .. }| Url::parse(expanded_url))
                            .collect::<Vec<_>>()
                    }
                    let urls = match &entities {
                        Some(FullTextEntities {
                            urls: Some(urls), ..
                        }) => to_expanded_urls(urls),
                        _ => Default::default(),
                    };
                    author_id
                        .and_then(get_user)
                        .map(|twitter_v2::User { username, id, .. }| {
                            Tweet::new(text.to_owned(), urls, username.to_owned(), id.as_u64())
                        })
                },
            )
            .collect::<Vec<_>>();
        Ok(tweets)
    }
}
