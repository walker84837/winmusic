use poise::serenity_prelude as serenity;
use songbird::SerenityInit;

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![play()],
            ..Default::default()
        })
        .token("YOUR_BOT_TOKEN_HERE")
        .intents(serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_VOICE_STATES)
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("{} is connected!", ready.user.name);
                Ok(())
            })
        });

    framework.run().await.unwrap();
}

#[poise::command(slash_command)]
async fn play(
    ctx: poise::Context<'_>,
    #[description = "YouTube URL"] url: String,
) -> Result<(), serenity::Error> {
    let guild_id = ctx.guild_id().ok_or_else(|| serenity::Error::Other("Not in a guild"))?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or_else(|| serenity::Error::Other("Songbird Voice client placed in at initialization."))?;

    let handler = manager.get(guild_id).ok_or_else(|| serenity::Error::Other("Not in a voice channel"))?;

    let source = songbird::ytdl(&url).await.map_err(|e| serenity::Error::Other("Error sourcing ffmpeg"))?;
    handler.lock().await.play_source(source);

    ctx.say("Playing song").await?;
    Ok(())
}
