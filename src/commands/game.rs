use std::{collections::HashMap, time::Duration};

use poise::{
    command,
    serenity_prelude::{
        futures::StreamExt,
        ButtonStyle,
        Color,
        ComponentInteraction,
        ComponentInteractionCollector,
        CreateActionRow,
        CreateButton,
        CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
        Member,
        Mentionable,
    },
    CreateReply,
};

use crate::{
    checks::{is_game_moderator, needs_active_game},
    game::Game,
    has_role,
    CmdRet,
    Context,
    Error,
    DEFAULT_COLOR,
};

fn get_remaining_lives_string(number_of_lives: i32) -> String {
    format!("{number_of_lives} ❤")
}

has_role!(has_mod_role, 1282277932798312513);

#[command(slash_command, rename = "start-game", guild_only, check = has_mod_role)]
pub async fn start_game(
    ctx: Context<'_>,
    #[description = "Der Moderator des Spiels"] moderator: Member,
) -> CmdRet {
    if ctx.data().game.lock().await.is_some() {
        prompt_override_game(ctx, moderator).await
    } else {
        create_new_game(ctx, moderator, None).await
    }
}

pub async fn prompt_override_game(ctx: Context<'_>, moderator: Member) -> CmdRet {
    let ctx_id = ctx.id().to_string();

    let (yes_id, no_id) = (format!("{ctx_id}_yes"), format!("{ctx_id}_no"));

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Es gibt ein laufendes Spiel.\nMöchtest du es überschreiben?")
                    .color(Color::RED),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new(&no_id)
                    .label("Nein")
                    .style(ButtonStyle::Danger),
                CreateButton::new(&yes_id)
                    .label("Ja")
                    .style(ButtonStyle::Success),
            ])]),
    )
    .await?;

    let mut collector = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(60))
        .filter(move |interaction| interaction.data.custom_id.starts_with(&ctx_id))
        .stream();

    if let Some(press) = collector.next().await {
        match &press.data.custom_id {
            id if id == &yes_id => create_new_game(ctx, moderator, Some(press)).await?,
            _ => {
                press
                    .create_response(
                        ctx,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .embed(
                                    CreateEmbed::default()
                                        .description("Abgebrochen")
                                        .color(Color::RED),
                                )
                                .components(vec![]),
                        ),
                    )
                    .await?;
            },
        };
    }

    Ok(())
}

pub async fn create_new_game(
    ctx: Context<'_>,
    moderator: Member,
    edit_on: Option<ComponentInteraction>,
) -> CmdRet {
    let mut game = ctx.data().game.lock().await;
    let mut vote = ctx.data().voting.lock().await;

    let embed = CreateEmbed::default()
        .title("Spiel gestartet")
        .description(format!(
            "✅ Das Spiel wurde mit {} als Moderator wurde erfolgreich gestartet",
            moderator.mention()
        ))
        .color(DEFAULT_COLOR);

    *game = Some(Game {
        moderator,
        creator: *ctx.interaction.member.as_ref().unwrap().clone(),
        members: HashMap::new(),
    });

    *vote = None;

    if let Some(interaction) = edit_on {
        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .components(vec![]),
                ),
            )
            .await?;
    } else {
        ctx.send(CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}

#[command(slash_command, rename = "add-user", guild_only, check = needs_active_game, check = is_game_moderator)]
pub async fn add_user(
    ctx: Context<'_>,
    #[description = "Der User der hinzugefügt werden soll"]
    #[rename = "user"]
    member: Member,
    lives: Option<i32>,
) -> CmdRet {
    {
        let mut lock = ctx.data().game.lock().await;
        let game = lock.as_mut().unwrap();

        if game.contains_player(&member.user) {
            return Err(format!("{} ist bereits im Spiel.", member.mention()).into());
        }

        game.members.insert(member.user.id, lives.unwrap_or(3));
    }

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .description(format!("➕ User {} wurde hinzugefügt", member.mention()))
                .color(DEFAULT_COLOR),
        ),
    )
    .await?;
    Ok(())
}

#[command(slash_command, rename = "remove-user", guild_only, check = is_game_moderator, check = needs_active_game)]
pub async fn remove_user(
    ctx: Context<'_>,
    #[description = "Der User der entfernt werden soll"]
    #[rename = "user"]
    member: Member,
) -> CmdRet {
    {
        let mut lock = ctx.data().game.lock().await;
        let game = lock.as_mut().unwrap();

        if !game.contains_player(&member.user) {
            return Err(format!("{} ist nicht im Spiel.", member.mention()).into());
        }

        game.members.remove(&member.user.id);
    }

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .description(format!("➖ User {} wurde entfernt", member.mention()))
                .color(DEFAULT_COLOR),
        ),
    )
    .await?;
    Ok(())
}

#[command(slash_command, rename = "show-game", guild_only, check = needs_active_game)]
pub async fn show_game(ctx: Context<'_>) -> CmdRet {
    let lock = ctx.data().game.lock().await;
    let game = lock.as_ref().expect("Expected an active game");

    let users = &game.members;

    let mut description = String::new();

    if users.is_empty() {
        description.push_str("Es sind keine User in diesem Spiel")
    } else {
        for (user, lives) in users {
            let user = user.to_user(ctx).await?;
            if *lives == 0 {
                description.push_str(
                    format!(
                        "~~{} ({})\n\n~~",
                        user.mention(),
                        get_remaining_lives_string(*lives)
                    )
                    .as_str(),
                )
            } else {
                description.push_str(
                    format!(
                        "{} ({})\n\n",
                        user.mention(),
                        get_remaining_lives_string(*lives)
                    )
                    .as_str(),
                )
            }
        }
    }

    let embed = CreateEmbed::default()
        .description(description)
        .color(DEFAULT_COLOR);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[command(slash_command, rename = "end-game", guild_only, check = has_mod_role, check = needs_active_game)]
pub async fn end_game(ctx: Context<'_>) -> CmdRet {
    let ctx_id = ctx.id().to_string();

    let (yes_id, no_id) = (format!("{ctx_id}_yes"), format!("{ctx_id}_no"));

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::default()
                    .description("Möchtest du das laufende Spiel wirklich beenden?")
                    .color(Color::RED),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new(&no_id)
                    .label("Nein")
                    .style(ButtonStyle::Danger),
                CreateButton::new(&yes_id)
                    .label("Ja")
                    .style(ButtonStyle::Success),
            ])]),
    )
    .await?;

    let mut collector = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(60))
        .filter(move |interaction| interaction.data.custom_id.starts_with(&ctx_id))
        .stream();

    if let Some(press) = collector.next().await {
        match &press.data.custom_id {
            id if id == &yes_id => {
                {
                    let mut game = ctx.data().game.lock().await;
                    *game = None;
                }

                press
                    .create_response(
                        ctx,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .embed(
                                    CreateEmbed::default()
                                        .description("Das Spiel wurde erfolgreich beendet")
                                        .color(DEFAULT_COLOR),
                                )
                                .components(vec![]),
                        ),
                    )
                    .await?;
            },
            _ => {
                press
                    .create_response(
                        ctx,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .embed(
                                    CreateEmbed::default()
                                        .description("Abgebrochen")
                                        .color(Color::RED),
                                )
                                .components(vec![]),
                        ),
                    )
                    .await?;
            },
        };
    }
    Ok(())
}
