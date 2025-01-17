use poise::serenity_prelude as serenity;
use reqwest::Client;
use songbird::input::YoutubeDl;
use std::sync::Arc;

struct Data {
    http_client: Arc<Client>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;

type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    let token = "YOUR_BOT_TOKEN_HERE";

    let framework = poise::Framework::builder()
        .setup(|_, _, _| {
            Box::pin(async move {
                // construct user data here (invoked when bot connects to Discord)
                Ok(())
            })
        })
        // Most configuration is done via the `FrameworkOptions` struct, which you can define with
        // a struct literal (hint: use `..Default::default()` to fill uninitialized
        // settings with their default value):
        .options(poise::FrameworkOptions {
            commands: vec![join(), play()],
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new("...", serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

#[poise::command(slash_command)]
async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().ok_or("Failed to get guild")?;
    let guild_id = guild.id;
    let user_id = ctx.author().id;

    let voice_state = guild
        .voice_states
        .get(&user_id)
        .ok_or("User is not in a voice channel")?;
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
async fn play(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let call = manager.get(guild_id).unwrap();

    let source = YoutubeDl::new(ctx.data().http_client.clone(), url);

    call.lock().await.enqueue_input(source.into());
    ctx.say("Now playing!").await?;
    Ok(())
}
