use std::{collections::HashMap, sync::Mutex, sync::Arc};

use crate::conf::Conf;

#[derive(Debug)]
pub struct State {
    pub conf: Arc<Conf>,
    pub verifiers: Mutex<HashMap<String, String>>,
}

impl State {
    pub fn new(conf: Arc<Conf>) -> State {
        State {
            conf,
            verifiers: Mutex::new(HashMap::new()),
        }
    }
}
