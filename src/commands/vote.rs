use std::{collections::HashMap, time::Duration};

use itertools::Itertools;
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
        UserId,
    },
    CreateReply,
};

use crate::{
    checks::{
        author_is_alive,
        did_not_vote,
        is_game_moderator,
        is_in_game,
        needs_active_game,
        needs_active_voting,
    },
    game::Voting,
    CmdRet,
    Context,
    Error,
    DEFAULT_COLOR,
};

type MemberVoteCount = HashMap<UserId, i32>;

#[command(slash_command, rename = "start-voting", guild_only, check = needs_active_game, check = is_game_moderator)]
pub async fn start_voting(ctx: Context<'_>) -> CmdRet {
    let creator = ctx.interaction.member.as_ref().unwrap();
    if ctx.data().voting.lock().await.is_some() {
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
                CreateEmbed::default()
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

pub async fn create_new_vote(
    ctx: Context<'_>,
    creator: &Member,
    edit_on: Option<ComponentInteraction>,
) -> CmdRet {
    let mut active_voting = ctx.data().voting.lock().await;

    let embed = CreateEmbed::default()
        .title("Vote gestartet")
        .description("🕛 Das Voting wurde gestartet.\nMan kann absofort voten.")
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

#[command(slash_command, guild_only, check = needs_active_game, check = needs_active_voting, check = is_in_game, check = did_not_vote, check = author_is_alive)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "Den User, den du voten willst"]
    #[rename = "user"]
    member: Member,
) -> CmdRet {
    {
        let mut active_voting = ctx.data().voting.lock().await;
        let active_voting = active_voting.as_mut().unwrap();

        let mut lock = ctx.data().game.lock().await;
        let game = lock.as_mut().unwrap();

        match game.members.get(&member.user.id) {
            // user dead
            Some(&hp) if (hp <= 0) => {
                return Err(format!("❌ {} ist ausgeschieden.", member.mention()).into())
            },
            // user not in game
            None => return Err(format!("{} ist nicht im Spiel.", member.mention()).into()),

            _ => (),
        };

        active_voting.map.insert(ctx.author().id, member.user.id);
    }

    let embed = CreateEmbed::default()
        .title("Voting")
        .description(format!("✅ {} hat gevotet.", ctx.author().mention()))
        .color(DEFAULT_COLOR);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

enum VoteOutcome {
    NoClearWinner {
        members_with_equal_votes_count: i32,
        max_vote_count: i32,
    },
    ClearWinner {
        user: UserId,
        num_votes: i32,
    },
}

#[command(slash_command, rename = "end-voting", guild_only, check = needs_active_voting, check = needs_active_game, check = is_game_moderator)]
pub async fn end_voting(ctx: Context<'_>) -> CmdRet {
    let mut voting = ctx.data().voting.lock().await;

    let mut lock = ctx.data().game.lock().await;
    let game = lock.as_mut().expect("Expected an active game");

    let (votes, mut who_voted_who_description) =
        sum_up_votes(&voting.as_ref().unwrap().map).await?;

    let mut member_died_embed: Option<CreateEmbed> = None;

    match decide_winner(&votes) {
        VoteOutcome::ClearWinner { user, num_votes } => {
            // ...and remove 1 hp from them
            game.members.entry(user).and_modify(|hp| *hp -= 1);
            let member = ctx
                .guild_id()
                .expect("guild ID should be set")
                .member(ctx, user)
                .await?;

            who_voted_who_description.push_str(&format!(
                "**{member} hat mit `{num_votes}` die meisten votes und verliert ein Leben!**"
            ));

            // check if the member that lost a life 'died' this round
            if game.is_player_dead(user)? {
                member_died_embed = Some(
                    CreateEmbed::default()
                        .description(format!("{member} ist ausgeschieden."))
                        .color(DEFAULT_COLOR),
                )
            }
        },
        VoteOutcome::NoClearWinner {
            members_with_equal_votes_count,
            max_vote_count,
        } => who_voted_who_description.push_str(&format!(
            "**{} Leute haben mit {} gleich viele Votes - Gleichstand!**",
            members_with_equal_votes_count, max_vote_count
        )),
    }

    let reply = create_end_voting_response(who_voted_who_description, &votes, member_died_embed);
    ctx.send(reply).await?;

    *voting = None;
    Ok(())
}

/// Sums up all votes of a specific member by providing a member->member map
async fn sum_up_votes(
    member_to_member_votes: &HashMap<UserId, UserId>,
) -> Result<(MemberVoteCount, String), Error> {
    let mut votes = HashMap::new();

    let mut who_voted_who_description = String::new();
    // Create vote `Member -> Amount of Votes` mapping
    for (voter, voted) in member_to_member_votes {
        votes.entry(*voted).and_modify(|num| *num += 1).or_insert(1);

        who_voted_who_description.push_str(&format!(
            "{} hat {} gevotet!\n\n",
            voter.mention(),
            voted.mention()
        ))
    }

    Ok((votes, who_voted_who_description))
}

fn decide_winner(votes: &MemberVoteCount) -> VoteOutcome {
    let max_num_of_votes = *votes.values().max_by_key(|n| **n).unwrap();
    let n = votes
        .values()
        .filter(|num| **num == max_num_of_votes)
        .count();

    if n > 1 {
        VoteOutcome::NoClearWinner {
            members_with_equal_votes_count: n as i32,
            max_vote_count: max_num_of_votes,
        }
    } else {
        let (user, num_votes) = votes
            .iter()
            .max_by_key(|(_, amount_votes)| *amount_votes)
            .map(|(user_id, num)| (user_id.to_owned(), num.to_owned()))
            .unwrap();

        VoteOutcome::ClearWinner { user, num_votes }
    }
}

fn get_voting_count_embed(votes: &HashMap<UserId, i32>) -> CreateEmbed {
    let mut description = String::new();

    for (user, amount_votes) in votes.iter().sorted_by(|a, b| a.1.cmp(b.1)) {
        description.push_str(&format!("`[{:>2}]` - {}\n", amount_votes, user.mention()));
    }

    CreateEmbed::default()
        .title("Anzahl der Votes")
        .description(description)
        .color(DEFAULT_COLOR)
}

fn create_end_voting_response(
    who_voted_who_description: String,
    votes: &HashMap<UserId, i32>,
    member_died_embed: Option<CreateEmbed>,
) -> CreateReply {
    let mut reply = CreateReply::default();
    // overview - who voted which person?
    reply = reply.embed(
        CreateEmbed::default()
            .title("Voting ist zuende.")
            .description(who_voted_who_description)
            .color(DEFAULT_COLOR),
    );

    // overview of all votes
    reply = reply.embed(get_voting_count_embed(votes));

    // additional info whether a member died in this round
    if let Some(member_died_embed) = member_died_embed {
        reply = reply.embed(member_died_embed);
    }
    reply
}
