use poise::{
    command,
    serenity_prelude::{CreateEmbed, Member, Mentionable},
    CreateReply,
};

use crate::{
    checks::{is_game_moderator, needs_active_game},
    CmdRet,
    Context,
    DEFAULT_COLOR,
};

#[command(slash_command, rename = "set-lives", guild_only, check = needs_active_game, check = is_game_moderator)]
pub async fn set_lives(ctx: Context<'_>, #[rename = "user"] member: Member, amount: i32) -> CmdRet {
    let mut lock = ctx.data().game.lock().await;
    let game = lock.as_mut().unwrap();

    game.set_player_health(ctx.author(), amount)?;

    let embed = CreateEmbed::default()
        .title("User wurde geupdated.")
        .description(format!(
            ":pencil2: {} hat nun `{}` Leben.",
            member.mention(),
            amount
        ))
        .color(DEFAULT_COLOR);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
