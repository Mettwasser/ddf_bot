use crate::{ContextEnum, Error};

const NO_ACTIVE_GAME: &str = "Es gibt kein aktives Spiel.";

pub async fn needs_active_game(ctx: ContextEnum<'_>) -> Result<bool, Error> {
    match *ctx.data().game.lock().await {
        Some(_) => Ok(true),
        None => Err(NO_ACTIVE_GAME.into()),
    }
}

pub async fn needs_active_voting(ctx: ContextEnum<'_>) -> Result<bool, Error> {
    match &*ctx.data().voting.lock().await {
        Some(_) => Ok(true),
        None => Err("Es gibt kein aktives Voting.".into()),
    }
}

/// This assumes an active voting
pub async fn did_not_vote(ctx: ContextEnum<'_>) -> Result<bool, Error> {
    let lock = ctx.data().voting.lock().await;
    let voting = lock.as_ref().expect("Expected an active voting");

    if voting.map.contains_key(&ctx.author().id) {
        Err("Du hast schon gevotet.".into())
    } else {
        Ok(true)
    }
}

/// This assumes an active game
pub async fn is_in_game(ctx: ContextEnum<'_>) -> Result<bool, Error> {
    let lock = ctx.data().game.lock().await;
    let game = lock.as_ref().expect("Expected an active game");

    if !game.members.contains_key(&ctx.author().id) {
        Err("Du bist diesem Spiel nicht beigetreten.".into())
    } else {
        Ok(true)
    }
}

/// This check .unwraps the [Game::moderator] field, so it assumes an active game
pub async fn is_game_moderator(ctx: ContextEnum<'_>) -> Result<bool, Error> {
    let lock = ctx.data().game.lock().await;
    let game = lock.as_ref().expect("Expected an active game");

    if game.moderator.user.id == ctx.author().id {
        Ok(true)
    } else {
        Err("Du bist nicht der Moderator dieses Spiels.".into())
    }
}

/// Assumes an active game, an active voting, and that the player is in the game
pub async fn author_is_alive(ctx: ContextEnum<'_>) -> Result<bool, Error> {
    let lock = ctx.data().game.lock().await;
    let game = lock.as_ref().expect("Expected an active game");

    if *game
        .members
        .get(&ctx.author().id)
        .expect("Expected user to be available in `author_is_alive`")
        > 0
    {
        Ok(true)
    } else {
        Err("Du bist ausgeschieden.".into())
    }
}
