use std::collections::HashMap;

use poise::serenity_prelude::{Member, User, UserId};

pub struct Game {
    pub creator: Member,
    pub moderator: Member,
    // user id to member's lives
    pub members: HashMap<UserId, i32>,
}

impl Game {
    pub fn contains_player(&self, player: &User) -> bool {
        self.members.contains_key(&player.id)
    }
}

pub struct Voting {
    pub creator: Member,
    // user id to member's lives
    pub map: HashMap<UserId, UserId>,
}
