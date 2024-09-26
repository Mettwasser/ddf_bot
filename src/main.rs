use std::{collections::HashSet, sync::Arc};

use ddf_bot::{
    commands::{
        game::{add_user, show_game, start_game},
        vote::{end_voting, start_voting, vote},
    },
    data::Data,
    error::handle_error,
    Error,
};
use poise::{
    serenity_prelude::{ClientBuilder, GatewayIntents, UserId},
    FrameworkError,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .init();

    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = GatewayIntents::privileged().difference(GatewayIntents::MESSAGE_CONTENT);

    let data = Arc::new(Data::default());

    let data_clone = data.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                show_game(),
                start_game(),
                add_user(),
                start_voting(),
                vote(),
                end_voting(),
            ],
            on_error: |err: FrameworkError<'_, Arc<Data>, Error>| Box::pin(handle_error(err)),
            owners: HashSet::from_iter([UserId::new(350749990681051149)]),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(data_clone)
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();

    Ok(())
}
