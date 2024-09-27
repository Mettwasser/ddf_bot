use {
    crate::{game::Game, has_role, CmdRet, Context, Error, DEFAULT_COLOR},
    poise::{
        command,
        serenity_prelude::{
            futures::StreamExt, ButtonStyle, Color, ComponentInteraction,
            ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
            CreateInteractionResponse, CreateInteractionResponseMessage, Member, Mentionable,
        },
        CreateReply,
    },
    std::{collections::HashMap, time::Duration},
};

fn get_remaining_lives_string(number_of_lives: i32) -> String {
    format!("{number_of_lives} ❤")
}

has_role!(has_mod_role, 1282277932798312513);

pub async fn needs_active_game(ctx: Context<'_>) -> Result<bool, Error> {
    match &*ctx.data().game.lock().await {
        Some(_) => Ok(true),
        None => Err("Es ist kein aktives Spiel vorhanden".into()),
    }
}

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

    let yes_no_id = (format!("{ctx_id}_yes"), format!("{ctx_id}_no"));

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .description("Es gibt ein laufendes Spiel.\nMöchtest du es überschreiben?")
                    .color(Color::RED),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new(&yes_no_id.1)
                    .label("Nein")
                    .style(ButtonStyle::Danger),
                CreateButton::new(&yes_no_id.0)
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
            id if id == &yes_no_id.0 => create_new_game(ctx, moderator, Some(press)).await?,
            _ => {
                press
                    .create_response(
                        ctx,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .embed(
                                    CreateEmbed::new()
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
    let Context::Application(ctx) = ctx else {
        unreachable!()
    };
    let mut game = ctx.data().game.lock().await;

    let embed = CreateEmbed::new()
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

#[command(slash_command, rename = "add-user", guild_only, check = has_mod_role, check = needs_active_game)]
pub async fn add_user(
    ctx: Context<'_>,
    #[description = "Der Benutzer der hinzugefügt werden soll"]
    #[rename = "user"]
    member: Member,
    lives: Option<i32>,
) -> CmdRet {
    ctx.data()
        .game
        .lock()
        .await
        .as_mut()
        .unwrap()
        .members
        .insert(member.user.id, lives.unwrap_or(3));

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .description(format!("➕ Nutzer {} wurde hinzugefügt", member.mention()))
                .color(DEFAULT_COLOR),
        ),
    )
    .await?;
    Ok(())
}

#[command(slash_command, rename = "show-game", guild_only, check = needs_active_game)]
pub async fn show_game(ctx: Context<'_>) -> CmdRet {
    let game = ctx.data().game.lock().await;
    let users = &game.as_ref().unwrap().members;

    let mut description = String::new();

    if users.is_empty() {
        description.push_str("Es sind keine Nutzer in diesem Spiel")
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

    let embed = CreateEmbed::new()
        .title("Das aktuelle Spiel")
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
                CreateEmbed::new()
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
                                    CreateEmbed::new()
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
                                    CreateEmbed::new()
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
