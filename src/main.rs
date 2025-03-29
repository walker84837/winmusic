use clap::Parser;
use log::info;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use songbird::SerenityInit;
use std::{path::Path, sync::Arc};

mod args;
mod commands;
mod config;
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

    let config = Config::new(Path::new(&args.config))?;
    let bot_config = Arc::new(config);

    let bot_config_clone = bot_config.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![join(), play(), skip(), stop(), pause(), resume()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    http_client: Client::new(),
                    config: bot_config,
                })
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .register_songbird()
        .framework(framework)
        .status(bot_config_clone.status)
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
