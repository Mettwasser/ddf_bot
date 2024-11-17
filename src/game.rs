use std::collections::HashMap;

use poise::serenity_prelude::{Member, Mentionable, UserId};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
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
    pub fn contains_player(&self, player: UserId) -> bool {
        self.members.contains_key(&player)
    }

    pub fn add_player(&mut self, player: UserId, health: i32) -> Result<(), PlayerError> {
        if self.members.contains_key(&player) {
            return Err(PlayerError::PlayerAlreadyAdded(player));
        }

        self.members.entry(player).or_insert(health);
        Ok(())
    }

    pub fn remove_player(&mut self, player: UserId) -> Result<(), PlayerError> {
        self.members
            .remove(&player)
            .map(|_| ())
            .ok_or(PlayerError::PlayerNotInGame(player))
    }

    pub fn set_player_health(&mut self, player: UserId, health: i32) -> Result<(), PlayerError> {
        if let Some(player_health) = self.members.get_mut(&player) {
            *player_health = health;
            Ok(())
        } else {
            Err(PlayerError::PlayerNotInGame(player))
        }
    }

    pub fn is_player_dead(&self, player: UserId) -> Result<bool, PlayerError> {
        if let Some(player_health) = self.members.get(&player) {
            Ok(*player_health <= 0)
        } else {
            Err(PlayerError::PlayerNotInGame(player))
        }
    }

    pub fn is_player_alive(&self, player: UserId) -> Result<bool, PlayerError> {
        self.is_player_dead(player).map(|is_alive| !is_alive)
    }
}

pub struct Voting {
    pub creator: Member,
    // user id to member's lives
    pub map: HashMap<UserId, UserId>,
}
