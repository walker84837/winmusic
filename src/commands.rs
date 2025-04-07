use crate::types::*;
use poise::serenity_prelude as serenity;
use serenity::{
    ComponentInteractionDataKind, CreateActionRow, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use songbird::input::{Compose, YoutubeDl};

/// Joins the user's voice channel
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

/// Plays a track and adds it to the queue
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

    // Determine whether the input is a URL or a search query
    let url = if is_url(&input) {
        input.clone()
    } else {
        let mut yt = YoutubeDl::new_search(ctx.data().http_client.clone(), input);
        let results = yt.search(Some(1)).await?;
        let result = results.into_iter().next().ok_or("No results found")?;
        result.source_url.ok_or("Result has no URL")?
    };

    let mut source = YoutubeDl::new(ctx.data().http_client.clone(), url.clone());
    let aux_meta = source.aux_metadata().await?;
    let track_title = aux_meta
        .title
        .clone()
        .unwrap_or_else(|| "Unknown Title".to_string());

    let input_source = source.into();
    let mut handler = call.lock().await;
    let track_handle = handler.enqueue_input(input_source).await;

    let uuid = track_handle.uuid();
    ctx.data().data.insert(uuid, aux_meta);

    ctx.say(format!("Now playing: {}", track_title)).await?;
    Ok(())
}

/// Skips the current track, and plays the next track
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

/// Pauses the current track
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

/// Stops the current track, and clears the queue
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

/// Search for a track on YouTube
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
        let url = meta.source_url.ok_or("No URL found")?;
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
                .ok_or("In this context, the bot isn't in a guild")?;
            let manager = songbird::get(ctx.serenity_context())
                .await
                .ok_or("Failed to get Songbird manager")?;
            let call = manager.get(guild_id).ok_or("Not in a voice channel")?;

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

/// Resumes the current track
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
    url::Url::parse(s.as_ref()).is_ok()
}

/// Displays the current playback status, including the queue
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?;
    let call = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let handler = call.lock().await;
    let queue = handler.queue();

    let mut response = String::new();

    if let Some(current) = handler.queue().current() {
        let uuid = current.uuid();
        let title = ctx
            .data()
            .data
            .get(&uuid)
            .and_then(|m| m.title.clone()) // Cloned the title
            .unwrap_or_else(|| "Unknown Title".to_string());
        response.push_str(&format!("**Now Playing:** {}\n", title));
    } else {
        response.push_str("No track is currently playing.\n");
    }

    let upcoming = queue.current_queue();
    if upcoming.is_empty() {
        response.push_str("The queue is empty.");
    } else {
        response.push_str("**Upcoming Tracks:**\n");
        for (i, track) in upcoming.iter().enumerate() {
            let uuid = track.uuid();
            let title = ctx
                .data()
                .data
                .get(&uuid)
                .and_then(|m| m.title.clone())
                .unwrap_or_else(|| "Unknown Title".to_string());
            response.push_str(&format!("{}. {}\n", i + 1, title));
        }
    }

    ctx.say(response).await?;
    Ok(())
}

/// Leaves the user's voice channel
#[poise::command(slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().ok_or("Failed to get guild")?.clone();
    let guild_id = guild.id;
    // let user_id = ctx.author().id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?
        .clone();

    manager.leave(guild_id).await?;
    ctx.say("Joined the voice channel!").await?;
    Ok(())
}
