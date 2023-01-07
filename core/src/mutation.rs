use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DbConn, Set};

#[derive(Clone, Debug)]
pub struct Mutation(Arc<DbConn>);

impl Mutation {
    pub fn new(conn: Arc<DbConn>) -> Mutation {
        Mutation(conn)
    }
}

impl Deref for Mutation {
    type Target = Arc<DbConn>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Mutation {
    pub async fn create_user(&self, name: String) -> Result<entity::user::ActiveModel> {
        let model = entity::user::ActiveModel {
            name: Set(name),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        }
        .save(self.as_ref())
        .await?;
        Ok(model)
    }
}
