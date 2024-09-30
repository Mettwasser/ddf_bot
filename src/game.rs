use std::collections::HashMap;

use poise::serenity_prelude::{Member, Mentionable, User, UserId};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlayerError {
    #[error("{} ist bereits im Spiel.", _0.mention())]
    PlayerAlreadyAdded(UserId),

    #[error("{} ist nicht im Spiel.", _0.mention())]
    PlayerNotInGame(UserId),
}

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

    pub fn add_player(&mut self, player: &User, health: i32) -> Result<(), PlayerError> {
        if self.members.contains_key(&player.id) {
            return Err(PlayerError::PlayerAlreadyAdded(player.id));
        }

        self.members.entry(player.id).or_insert(health);
        Ok(())
    }

    pub fn remove_player(&mut self, player: &User) -> Result<(), PlayerError> {
        self.members
            .remove(&player.id)
            .map(|_| ())
            .ok_or(PlayerError::PlayerNotInGame(player.id))
    }

    pub fn set_player_health(&mut self, player: &User, health: i32) -> Result<(), PlayerError> {
        if let Some(player_health) = self.members.get_mut(&player.id) {
            *player_health = health;
            Ok(())
        } else {
            Err(PlayerError::PlayerNotInGame(player.id))
        }
    }
}

pub struct Voting {
    pub creator: Member,
    // user id to member's lives
    pub map: HashMap<UserId, UserId>,
}
