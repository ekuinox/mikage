use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "twitter_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[sea_orm(auto_increment)]
    pub user_id: i64,
    pub screen_name: String,
    pub display_name: String,
    pub avatar_url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub owner_user_id: i64, // User::Id
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
