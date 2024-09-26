use {
    crate::{
        commands::{
            game::NeedsActiveGameError,
            vote::{AlreadyVotedError, NeedsActiveVotingError, NotInGameError},
        },
        utils::MissingRole,
        Context, Error, FrameworkError,
    },
    poise::{
        serenity_prelude::{self, Color, CreateEmbed, Mentionable, RoleId},
        CreateReply,
    },
    thiserror::Error,
};

pub fn error_embed(description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title("Error")
        .description(description)
        .color(Color::RED)
}

pub async fn handle_error(err: FrameworkError<'_>) {
    tracing::error!(error = %err);

    use poise::FrameworkError::*;
    match err {
        Command { error, ctx, .. } => handle_command_error(error, ctx).await.unwrap(),
        CommandCheckFailed { error, ctx, .. } => {
            // Option, it's None if an `Ok(false)` is returned from a check
            // Unwrap, because we want every error to be handled this way
            handle_command_check_error(error.unwrap(), ctx)
                .await
                .unwrap()
        },
        err => poise::builtins::on_error(err).await.unwrap(),
    }
}

pub async fn handle_command_error(
    err: Error,
    ctx: Context<'_>,
) -> Result<(), serenity_prelude::Error> {
    poise::builtins::on_error(poise::FrameworkError::new_command(ctx, err))
        .await
        .unwrap();

    Ok(())
}

pub async fn handle_command_check_error(
    err: Error,
    ctx: Context<'_>,
) -> Result<(), serenity_prelude::Error> {
    if let Some(err) = err.downcast_ref::<NeedsActiveGameError>() {
        ctx.send(CreateReply::default().embed(error_embed(err.0)))
            .await?;
    } else if let Some(err) = err.downcast_ref::<MissingRole>() {
        ctx.send(CreateReply::default().embed(error_embed(format!(
            "Missing {} role",
            ctx.guild_id().unwrap().roles(ctx).await?.get(&RoleId::new(err.0)).unwrap().mention()
        ))))
        .await?;
    } else if let Some(err) = err.downcast_ref::<NeedsActiveVotingError>() {
        ctx.send(CreateReply::default().embed(error_embed(err.to_string())))
            .await?;
    } else if let Some(err) = err.downcast_ref::<AlreadyVotedError>() {
        ctx.send(CreateReply::default().embed(error_embed(err.to_string())))
            .await?;
    } else if let Some(err) = err.downcast_ref::<NotInGameError>() {
        ctx.send(CreateReply::default().embed(error_embed(err.to_string())))
            .await?;
    } else {
        poise::builtins::on_error(poise::FrameworkError::new_command(ctx, err)).await?;
    }

    Ok(())
}
