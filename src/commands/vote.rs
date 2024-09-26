use {
    super::game::has_mod_role,
    crate::{
        commands::game::needs_active_game,
        game::{Game, Voting},
        Error, DEFAULT_COLOR,
    },
    poise::serenity_prelude::{
        futures::StreamExt, ButtonStyle, Color, ComponentInteraction,
        ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage, Member, Mentionable, User,
        UserId,
    },
    std::{collections::HashMap, time::Duration},
    thiserror::Error,
};
use {crate::CmdRet, poise::command};
use {crate::Context, poise::CreateReply};

#[derive(Debug, Error)]
#[error("{0}")]
pub struct NeedsActiveVotingError(pub &'static str);

pub async fn needs_active_voting(ctx: Context<'_>) -> Result<bool, Error> {
    match &*ctx.data().active_voting.lock().await {
        Some(_) => Ok(true),
        None => Err(Box::new(NeedsActiveVotingError(
            "Es gibt kein aktives Voting",
        ))),
    }
}

pub fn mentioned_user_in_game_and_alive(game: &Game, mentioned_user: &User) -> Result<(), Error> {
    match game.members.get(&mentioned_user.id) {
        // user alive
        Some(&hp) if hp > 0 => Ok(()),
        // user dead
        Some(_) => Err(format!("❌ {} ist ausgeschieden!", mentioned_user.mention()).into()),
        // user not in game
        None => Err(format!("{} ist nicht im Spiel!", mentioned_user.mention()).into()),
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct AlreadyVotedError(pub &'static str);

/// Also checks for an active voting
pub async fn did_not_vote(ctx: Context<'_>) -> Result<bool, Error> {
    match &*ctx.data().active_voting.lock().await {
        Some(voting) => {
            if voting.map.contains_key(&ctx.author().id) {
                Err(Box::new(AlreadyVotedError("Du hast schon gevotet!")))
            } else {
                Ok(true)
            }
        },
        None => Err(Box::new(NeedsActiveVotingError(
            "Es gibt kein aktives Voting",
        ))),
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct NotInGameError(pub String);

/// Also checks for an active game
pub async fn is_in_game(ctx: Context<'_>) -> Result<bool, Error> {
    match &*ctx.data().game.lock().await {
        Some(game) => {
            if !game.members.contains_key(&ctx.author().id) {
                Err(Box::new(NotInGameError(
                    "Du bist diesem Spiel nicht beigetreten!".to_owned(),
                )))
            } else {
                Ok(true)
            }
        },
        None => Err(Box::new(NeedsActiveVotingError(
            "Es gibt kein aktives Voting",
        ))),
    }
}

#[command(slash_command, rename = "start-voting", guild_only, check = has_mod_role, check = needs_active_game)]
pub async fn start_voting(ctx: Context<'_>) -> CmdRet {
    let Context::Application(app_ctx) = &ctx else {
        unreachable!()
    };
    let creator = app_ctx.interaction.member.as_ref().unwrap();
    if ctx.data().active_voting.lock().await.is_some() {
        prompt_override_vote(ctx, creator).await
    } else {
        create_new_vote(ctx, creator, None).await
    }
}

pub async fn prompt_override_vote(ctx: Context<'_>, creator: &Member) -> CmdRet {
    let ctx_id = ctx.id().to_string();

    let yes_no_id = (format!("{ctx_id}_yes"), format!("{ctx_id}_no"));

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .description("Es gibt ein laufendes Voting.\nMöchtest du es überschreiben?")
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
            id if id == &yes_no_id.0 => create_new_vote(ctx, creator, Some(press)).await?,
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

pub async fn create_new_vote(
    ctx: Context<'_>,
    creator: &Member,
    edit_on: Option<ComponentInteraction>,
) -> CmdRet {
    let mut active_voting = ctx.data().active_voting.lock().await;

    let embed = CreateEmbed::new()
        .title("Vote gestartet")
        .description(format!(
            "🕛 Das Voting wurde mit {} als Moderator wurde erfolgreich gestartet.\nMan kann absofort voten!",
            creator.mention()
        ))
        .color(DEFAULT_COLOR);

    *active_voting = Some(Voting {
        creator: creator.clone(),
        map: HashMap::new(),
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

#[command(slash_command, guild_only, check = is_in_game, check = did_not_vote)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "Den User, den du voten willst"]
    #[rename = "user"]
    member: Member,
) -> CmdRet {
    {
        let mut active_voting = ctx.data().active_voting.lock().await;
        let active_voting = active_voting.as_mut().unwrap();

        let mut game = ctx.data().game.lock().await;
        let game = game.as_mut().unwrap();

        mentioned_user_in_game_and_alive(game, &member.user)?;
        active_voting.map.insert(ctx.author().id, member.user.id);
    }

    let embed = CreateEmbed::new()
        .title("Voting")
        .description(format!("✅ {} hat gevotet!", ctx.author().mention()))
        .color(DEFAULT_COLOR);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[command(slash_command, rename = "end-voting", guild_only, check = has_mod_role, check = needs_active_voting, check = needs_active_game)]
pub async fn end_voting(ctx: Context<'_>) -> CmdRet {
    let mut active_voting = ctx.data().active_voting.lock().await;
    let mut game = ctx.data().game.lock().await;

    let mut description = String::new();
    let mut votes: HashMap<UserId, i32> = HashMap::new();

    let mut reply = CreateReply::default();

    let guild = ctx.guild_id().unwrap();
    for (voter, voted) in &active_voting.as_ref().unwrap().map {
        let voter_member = guild.member(ctx, voter).await?;
        let voted_member = guild.member(ctx, voted).await?;

        votes.entry(*voted).and_modify(|num| *num += 1).or_insert(1);

        description.push_str(&format!(
            "{} hat {} gevotet!\n\n",
            voter_member.mention(),
            voted_member.mention()
        ))
    }

    let (voted_highest, _) = votes
        .into_iter()
        .max_by_key(|elem| elem.1)
        .ok_or("Es hat niemand gevotet!")?;

    game.as_mut()
        .unwrap()
        .members
        .entry(voted_highest)
        .and_modify(|hp| *hp -= 1);

    let member = guild.member(ctx, voted_highest).await?;

    description.push_str(&format!(
        "**{member} hat die meisten votes und verliert ein Leben!**"
    ));

    reply = reply.embed(
        CreateEmbed::new()
            .title("Voting ist zuende!")
            .description(description)
            .color(DEFAULT_COLOR),
    );

    if *game.as_ref().unwrap().members.get(&voted_highest).unwrap() <= 0 {
        reply = reply.embed(
            CreateEmbed::new()
                .description(format!("{member} is ausgeschieden!"))
                .color(DEFAULT_COLOR),
        )
    }
    ctx.send(reply).await?;

    *active_voting = None;
    Ok(())
}
