use anyhow::{bail, Result};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct OAuth2ClientCredentialDatabase {
    tree: sled::Tree,
}

macro_rules! tree_fields {
    ($struct:ty, $field:ident) => {
        paste::paste! {
            impl $struct {
                #[allow(unused)]
                pub fn $field(&self) -> Result<String> {
                    let Some(value) = self.tree.get(std::stringify!($field))? else {
                        bail!("{} not found", std::stringify!($field));
                    };
                    let value = String::from_utf8(value.to_vec())?;
                    Ok(value)
                }

                #[allow(unused)]
                pub fn [<set_ $field>](&self, value: &str) -> Result<()> {
                    let _ = self.tree.insert(std::stringify!($field), value.as_bytes())?;
                    Ok(())
                }

                #[allow(unused)]
                pub fn [<drop_ $field>](&self) -> Result<()> {
                    let _ = self.tree.remove(std::stringify!($field))?;
                    Ok(())
                }
            }
        }
    };
    ($struct:ty, [$($field:ident),+ $(,)?]) => {
        $(
            tree_fields!($struct, $field);
        )*
    }
}

impl OAuth2ClientCredentialDatabase {
    pub fn new(tree: sled::Tree) -> OAuth2ClientCredentialDatabase {
        OAuth2ClientCredentialDatabase { tree }
    }
}

tree_fields!(
    OAuth2ClientCredentialDatabase,
    [client_id, client_secret, access_token, refresh_token, pkce_verifier, csrf_state]
);

#[derive(Clone, Debug)]
pub struct Database {
    db: sled::Db,
}

impl Database {
    pub fn from_path(path: &Path) -> Result<Database> {
        let db = sled::open(path)?;
        let database = Database { db };
        Ok(database)
    }

    pub fn twitter_credential(&self) -> Result<OAuth2ClientCredentialDatabase> {
        let tree = self.db.open_tree("twitter_credentials")?;
        Ok(OAuth2ClientCredentialDatabase::new(tree))
    }

    pub fn spotify_credentials(&self) -> Result<OAuth2ClientCredentialDatabase> {
        let tree = self.db.open_tree("spotify_credentials")?;
        Ok(OAuth2ClientCredentialDatabase::new(tree))
    }
}
