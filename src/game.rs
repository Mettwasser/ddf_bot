use {
    poise::serenity_prelude::{Member, UserId},
    std::collections::HashMap,
};

pub struct Game {
    pub creator: Member,
    pub moderator: Member,
    // user id to member's lives
    pub members: HashMap<UserId, i32>,
}

pub struct Voting {
    pub creator: Member,
    // user id to member's lives
    pub map: HashMap<UserId, UserId>,
}
