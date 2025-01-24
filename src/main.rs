use clap::Parser;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use songbird::input::YoutubeDl;
use std::{path::Path, sync::Arc};

mod args;
mod config;
use crate::{args::Args, config::Config};

struct Data {
    http_client: Client,
    config: Arc<Config>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::try_parse()?;

    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect(
        "Discord bot token missing. Set DISCORD_TOKEN environment variable in your .env file.",
    );

    let bot_config = Arc::new(Config::new(&Path::new(&args.config))?);

    let bot_config_clone = bot_config.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![join(), play()],
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

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::GUILD_MESSAGES;
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .status(bot_config_clone.status)
        .await?;

    client.start().await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().ok_or("Failed to get guild")?.clone();
    let guild_id = guild.id;
    let user_id = ctx.author().id;

    let voice_state = guild
        .voice_states
        .get(&user_id)
        .ok_or("User is not in a voice channel")?
        .clone();
    let channel_id = voice_state
        .channel_id
        .ok_or("User is not in a voice channel")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?
        .clone();

    manager.join_gateway(guild_id, channel_id).await?;
    ctx.say("Joined the voice channel!").await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn play(
    ctx: Context<'_>,
    #[description = "URL of the song to play"] url: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?
        .clone();
    let call = manager.get(guild_id).ok_or("Not in a voice channel")?;

    let source = YoutubeDl::new(ctx.data().http_client, url);

    // call.lock().await.add_global_source("song", source.into())?;
    let a = call.lock().await;
    let b: songbird::Call = (&*a).clone();
    ctx.say("Now playing!").await?;
    Ok(())
}
