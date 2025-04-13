use clap::Parser;
use log::info;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use std::{path::Path, sync::Arc};

mod args;
mod commands;
mod config;
mod db;
mod types;
use crate::{args::Args, commands::*, config::Config, types::*};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect(
        "Discord bot token missing. Set DISCORD_TOKEN environment variable in your .env file.",
    );

    let config = Arc::new(Config::new(Path::new(&args.config))?);
    let config_for_setup = config.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                join(),
                leave(),
                play(),
                skip(),
                stop(),
                pause(),
                resume(),
                status(),
                search(),
                playlist(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::new(config_for_setup.clone()).await?)
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .register_songbird()
        .framework(framework)
        .status(config.status)
        .await?;

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for CTRL+C");
        info!("CTRL+C received, shutting down gracefully...");
        shard_manager.shutdown_all().await;
    });

    client.start().await?;
    Ok(())
}
