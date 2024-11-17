use std::collections::HashMap;

use poise::serenity_prelude::{Mentionable, UserId};

use crate::{Context, Error};

pub struct VoteSummary<'a>(&'a HashMap<UserId, UserId>);

impl<'a> VoteSummary<'a> {
    pub async fn try_to_string(self, ctx: Context<'_>) -> Result<String, Error> {
        let guild = ctx.guild_id().expect("guild ID should be set");
        let mut who_voted_who_description = String::new();

        let mut member_names = Vec::new();
        for user_id in self.0.keys() {
            member_names.push(guild.member(ctx, user_id).await?.display_name().len());
        }
        let max_member_name_len = member_names.into_iter().max().unwrap_or(0);

        // Create vote `Member -> Amount of Votes` mapping
        for (voter, voted) in self.0 {
            let voter_member = guild.member(ctx, voter).await?;
            let voted_member = guild.member(ctx, voted).await?;

            who_voted_who_description.push_str(&format!(
                "{:<width$} -> {}",
                voter_member.mention(),
                voted_member.mention(),
                width = max_member_name_len
            ))
        }

        Ok(who_voted_who_description)
    }
}
