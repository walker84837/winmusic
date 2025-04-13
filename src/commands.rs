use crate::types::*;
use poise::serenity_prelude as serenity;
use serenity::{
    ComponentInteractionDataKind, CreateActionRow, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use songbird::input::{AuxMetadata, Compose, YoutubeDl};
use spotify_rs::{
    ClientCredsFlow,
    auth::{NoVerifier, Token},
    client::Client,
    model::PlayableItem::Track,
};
use std::time::Duration;
use tokio::process::Command;

pub struct SpotifyTrack {
    pub name: String,
    pub artist: String,
}

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

/// Leaves the user's voice channel
#[poise::command(slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().ok_or("Failed to get guild")?.clone();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Failed to get Songbird manager")?
        .clone();

    manager.leave(guild_id).await?;
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

fn extract_playlist_id(url: &str) -> Result<&str, Error> {
    let uri_parts: Vec<&str> = url.split(':').collect();

    if uri_parts.len() == 3 && uri_parts[0] == "spotify" && uri_parts[1] == "playlist" {
        return Ok(uri_parts[2]);
    }

    let path_segments: Vec<&str> = url.split('/').collect();

    if let Some(index) = path_segments.iter().position(|&s| s == "playlist") {
        if let Some(id) = path_segments.get(index + 1) {
            let id = id.split('?').next().unwrap_or(id);
            return Ok(id);
        }
    }

    Err("Invalid Spotify playlist URL".into())
}

async fn fetch_spotify_playlist_tracks(
    spotify_client: &mut Client<Token, ClientCredsFlow, NoVerifier>,
    playlist_url: &str,
) -> Result<Vec<SpotifyTrack>, Error> {
    let playlist_id = extract_playlist_id(playlist_url)?;

    let mut offset = 0;
    let limit = 100;
    let mut tracks = Vec::new();

    loop {
        let page = spotify_client
            .playlist_items(&*playlist_id)
            .limit(limit)
            .offset(offset)
            .get()
            .await?;

        for item in page.items {
            if let Track(track) = item.track {
                let name = track.name;
                let artist = track
                    .artists
                    .first()
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown Artist".to_string());
                tracks.push(SpotifyTrack { name, artist });
            }
        }

        if page.next.is_none() {
            break;
        }
        offset += limit;
    }

    Ok(tracks)
}

async fn fetch_playlist_videos(playlist: impl AsRef<str>) -> Result<Vec<AuxMetadata>, Error> {
    let output = Command::new("yt-dlp")
        .args(&["--flat-playlist", "-j", playlist.as_ref()])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", stderr).into());
    }

    let mut metadata_list = Vec::new();

    // Process each JSON entry in the output
    for line in output
        .stdout
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
    {
        let entry: serde_json::Value = serde_json::from_slice(line)?;

        // Extract relevant fields from JSON response
        let title = entry
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let url = entry
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let duration = entry
            .get("duration")
            .and_then(|v| v.as_f64())
            .map(|d| Duration::from_secs_f64(d));

        let channel = entry
            .get("channel")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let thumbnail = entry
            .get("thumbnails")
            .and_then(|v| v.as_array())
            .and_then(|a| a.last())
            .and_then(|thumb| thumb.get("url"))
            .and_then(|url| url.as_str())
            .map(|s| s.to_string());

        let mut meta = AuxMetadata::default();
        meta.title = title;
        meta.source_url = url;
        meta.duration = duration;
        meta.channel = channel.clone();
        meta.artist = channel;
        meta.thumbnail = thumbnail;
        meta.start_time = Some(Duration::from_secs(0));

        metadata_list.push(meta);
    }

    Ok(metadata_list)
}

/// Play a playlist from a URL (Spotify or YouTube)
#[poise::command(slash_command)]
pub async fn playlist(
    ctx: Context<'_>,
    #[description = "Playlist URL"] query: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    if !is_url(&query) {
        return Err("Invalid URL".into());
    }

    let is_youtube = |url: &str| url.contains("youtube") || url.contains("youtu.be");
    let is_spotify = |url: &str| url.contains("spotify");

    if is_spotify(&query) {
        let playlist_id = extract_playlist_id(&query)?;
        let mut spotify_client = ctx.data().spotify_client.lock().await;
        let tracks = fetch_spotify_playlist_tracks(&mut *spotify_client, &query).await?;

        ctx.data()
            .database
            .add_playlist_tracks(playlist_id, &tracks)
            .await?;

        // For each Spotify track, build a search query and try to get the YouTube URL.
        let mut enqueued_tracks = 0;
        for track in tracks {
            let search_query = format!("{} {}", track.name, track.artist);
            let mut yt = YoutubeDl::new_search(ctx.data().http_client.clone(), search_query);
            let results = yt.search(Some(1)).await?;
            if let Some(result) = results.into_iter().next() {
                let yt_url = result.source_url.ok_or("No source URL found")?;

                // Enqueue the track
                enqueued_tracks += 1;
                ctx.say(format!(
                    "Enqueued track: {} by {} (from {})",
                    track.name, track.artist, yt_url
                ))
                .await?;
            } else {
                ctx.say(format!(
                    "No YouTube result for: {} by {}",
                    track.name, track.artist
                ))
                .await?;
            }
        }
        ctx.say(format!("Total tracks enqueued: {}", enqueued_tracks))
            .await?;
    } else if is_youtube(&query) {
        let videos = fetch_playlist_videos(&query).await?;
        for video in videos {
            println!("{:#?}", video);
        }
    }

    Ok(())
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
            .and_then(|m| m.title.clone())
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
