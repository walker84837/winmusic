use crate::types::*;
use poise::serenity_prelude as serenity;
use serenity::{
    ComponentInteractionDataKind, CreateActionRow, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use songbird::input::YoutubeDl;

#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
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

    manager.join(guild_id, channel_id).await?;
    ctx.say("Joined the voice channel!").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL or search query"] input: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?
        .clone();
    let call = manager.get(guild_id).ok_or("Not in a voice channel")?;

    let url = if is_url(&input) {
        input.clone()
    } else {
        let mut yt = YoutubeDl::new_search(ctx.data().http_client.clone(), input);
        let results = yt.search(Some(1)).await?;
        let result = results.into_iter().next().ok_or("No results found")?;
        result.source_url.ok_or("Result has no URL")?
    };

    let source = YoutubeDl::new(ctx.data().http_client.clone(), url);
    let input = source.into();

    let mut handler = call.lock().await;
    handler.enqueue_input(input).await;

    ctx.say("Added to queue!").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?;
    let call = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let handler = call.lock().await;
    let queue = handler.queue();
    queue.skip()?;
    ctx.say("Skipped current track.").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?;
    let call = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let handler = call.lock().await;
    let queue = handler.queue();
    queue.pause()?;
    ctx.say("Playback paused.").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?;
    let call = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let handler = call.lock().await;
    let queue = handler.queue();
    queue.stop();
    ctx.say("Stopped playback and cleared the queue.").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Search query"] query: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let mut yt = YoutubeDl::new_search(ctx.data().http_client.clone(), query);
    let results = yt.search(Some(5)).await?;

    let mut options = Vec::new();
    let mut choices = Vec::new();
    for (i, meta) in results.into_iter().enumerate() {
        let title = meta.title.unwrap_or_else(|| "Unknown Title".to_string());
        let url = meta.source_url.ok_or_else(|| "No URL found")?;
        choices.push((title.clone(), url.clone()));
        options.push(CreateSelectMenuOption::new(
            format!("{}: {}", i + 1, title),
            url,
        ));
    }

    let select_menu =
        CreateSelectMenu::new("search_results", CreateSelectMenuKind::String { options })
            .placeholder("Choose a track");
    let reply = poise::CreateReply::default()
        .content("Search Results:")
        .components(vec![CreateActionRow::SelectMenu(select_menu)]);

    let reply_handle = ctx.send(reply).await?;
    let message = reply_handle.message().await?;

    let interaction = message
        .await_component_interaction(ctx.serenity_context())
        .author_id(ctx.author().id)
        .await;

    if let Some(interaction) = interaction {
        if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
            let selected_url = &values[0];
            interaction.defer(ctx.serenity_context()).await?;

            let guild_id = ctx
                .guild_id()
                .ok_or_else(|| "In this context, the bot isn't in a guild")?;
            let manager = songbird::get(ctx.serenity_context())
                .await
                .ok_or_else(|| "Failed to get Songbird manager")?;
            let call = manager
                .get(guild_id)
                .ok_or_else(|| "Not in a voice channel")?;

            let source = YoutubeDl::new(ctx.data().http_client.clone(), selected_url.to_string());
            let input = source.into();

            let mut handler = call.lock().await;
            handler.enqueue_input(input).await;

            ctx.say(format!("Added: {}", selected_url)).await?;
        } else {
            return Err("Unexpected interaction data type".into());
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?;
    let call = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let handler = call.lock().await;
    let queue = handler.queue();
    queue.resume()?;
    ctx.say("Playback resumed.").await?;
    Ok(())
}

fn is_url(s: impl AsRef<str>) -> bool {
    let s = s.as_ref();
    url::Url::parse(s).is_ok()
}
