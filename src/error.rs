use poise::{
    serenity_prelude::{self, Color, CreateEmbed},
    CreateReply,
};

use crate::{Context, Error, FrameworkError, IntoAppContext};

pub fn error_embed(description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::default()
        .title("Error")
        .description(description)
        .color(Color::RED)
}

pub async fn handle_error(err: FrameworkError<'_>) {
    use poise::FrameworkError::*;
    match err {
        Command { error, ctx, .. } => {
            tracing::error!(error = %error);
            handle_command_error(error, ctx.into_app_context())
                .await
                .unwrap();
        },
        CommandCheckFailed { error, ctx, .. } => {
            // Option, it's None if an `Ok(false)` is returned from a check
            // Unwrap, because we want every error to be handled this way
            handle_command_check_error(error.unwrap(), ctx.into_app_context())
                .await
                .unwrap()
        },
        err => {
            tracing::error!(error = %err);
            poise::builtins::on_error(err).await.unwrap();
        },
    }
}

pub async fn handle_command_error(
    err: Error,
    ctx: Context<'_>,
) -> Result<(), serenity_prelude::Error> {
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Fehler")
                .description(err.to_string())
                .color(Color::DARK_RED),
        ),
    )
    .await?;

    Ok(())
}

pub async fn handle_command_check_error(
    err: Error,
    ctx: Context<'_>,
) -> Result<(), serenity_prelude::Error> {
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Fehler")
                .description(err.to_string())
                .color(Color::DARK_RED),
        ),
    )
    .await?;

    Ok(())
}
