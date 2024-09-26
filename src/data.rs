use {
    crate::game::{Game, Voting},
    tokio::sync::Mutex,
};

pub struct Data {
    pub game: Mutex<Option<Game>>,
    pub active_voting: Mutex<Option<Voting>>,
    pub votings: Mutex<Vec<Voting>>,
}

impl Data {
    pub fn new() -> Self {
        Data {
            game: Mutex::new(None),
            active_voting: Mutex::new(None),
            votings: Mutex::new(Vec::new()),
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}
