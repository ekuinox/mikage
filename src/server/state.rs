use std::{collections::HashMap, sync::Mutex, sync::Arc};

use sea_orm::DatabaseConnection;

use crate::conf::Conf;

#[derive(Debug)]
pub struct State {
    pub conf: Arc<Conf>,
    pub conn: Arc<DatabaseConnection>,
    pub verifiers: Mutex<HashMap<String, String>>,
}

impl State {
    pub fn new(conf: Arc<Conf>, conn: Arc<DatabaseConnection>) -> State {
        State {
            conf,
            conn,
            verifiers: Mutex::new(HashMap::new()),
        }
    }
}
