use std::{ops::Deref, sync::Arc};

use anyhow::{bail, Result};
use sea_orm::{DbConn, EntityTrait};

#[derive(Clone, Debug)]
pub struct Query(Arc<DbConn>);

impl Query {
    pub fn new(conn: Arc<DbConn>) -> Query {
        Query(conn)
    }
}

impl Deref for Query {
    type Target = Arc<DbConn>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Query {
    pub async fn get_user_by_id(&self, id: i32) -> Result<entity::user::ActiveModel> {
        let model = entity::user::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?;
        let Some(model) = model else {
            bail!("id={} not found.", id);
        };

        Ok(model.into())
    }
}
