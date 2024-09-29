use tokio::sync::Mutex;

use crate::game::{Game, Voting};

pub struct Data {
    pub game: Mutex<Option<Game>>,
    pub voting: Mutex<Option<Voting>>,
}

impl Data {
    pub fn new() -> Self {
        Data {
            game: Mutex::new(None),
            voting: Mutex::new(None),
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}
