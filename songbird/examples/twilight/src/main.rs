//! This example adapts Twilight's [basic lavalink bot] to use Songbird as its voice driver.
//!
//! # Twilight-rs attribution
//! ISC License (ISC)
//! 
//! Copyright (c) 2019, 2020 (c) The Twilight Contributors
//! 
//! Permission to use, copy, modify, and/or distribute this software for any purpose
//! with or without fee is hereby granted, provided that the above copyright notice
//! and this permission notice appear in all copies.
//! 
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
//! REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
//! FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
//! INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS
//! OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
//! TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF
//! THIS SOFTWARE.
//!
//!
//! [basic lavalink bot]: https://github.com/twilight-rs/twilight/tree/trunk/lavalink/examples/basic-lavalink-bot

use futures::StreamExt;
use std::{collections::HashMap, env, error::Error, future::Future, sync::Arc};
use songbird::{input::{Input, Restartable}, tracks::{PlayMode, TrackHandle}, Songbird};
use tokio::sync::RwLock;
use twilight_gateway::{Cluster, Event};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::Message, gateway::payload::MessageCreate, id::GuildId};
use twilight_standby::Standby;

#[derive(Clone, Debug)]
struct State {
    cluster: Cluster,
    http: HttpClient,
    trackdata: Arc<RwLock<HashMap<GuildId, TrackHandle>>>,
    songbird: Arc<Songbird>,
    standby: Standby,
}

fn spawn(
    fut: impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
) {
    tokio::spawn(async move {
        if let Err(why) = fut.await {
            tracing::debug!("handler error: {:?}", why);
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let state = {
        let token = env::var("DISCORD_TOKEN")?;

        let http = HttpClient::new(&token);
        let user_id = http.current_user().await?.id;

        let cluster = Cluster::new(token).await?;

        let shard_count = cluster.shards().len();
        let songbird = Songbird::twilight(cluster.clone(), shard_count as u64, user_id);

        cluster.up().await;

        State {
            cluster,
            http,
            trackdata: Default::default(),
            songbird,
            standby: Standby::new(),
        }
    };

    let mut events = state.cluster.events();

    while let Some(event) = events.next().await {
        state.standby.process(&event.1);
        state.songbird.process(&event.1).await;

        if let Event::MessageCreate(msg) = event.1 {
            if msg.guild_id.is_none() || !msg.content.starts_with('!') {
                continue;
            }

            match msg.content.splitn(2, ' ').next() {
                Some("!join") => spawn(join(msg.0, state.clone())),
                Some("!leave") => spawn(leave(msg.0, state.clone())),
                Some("!pause") => spawn(pause(msg.0, state.clone())),
                Some("!play") => spawn(play(msg.0, state.clone())),
                Some("!seek") => spawn(seek(msg.0, state.clone())),
                Some("!stop") => spawn(stop(msg.0, state.clone())),
                Some("!volume") => spawn(volume(msg.0, state.clone())),
                _ => continue,
            }
        }
    }

    Ok(())
}

async fn join(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the channel ID you want me to join?")?
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let channel_id = msg.content.parse::<u64>()?;

    let guild_id = msg.guild_id.ok_or("Can't join a non-guild channel.")?;

    let (_handle, success) = state
        .songbird
        .join(guild_id, channel_id)
        .await;

    let content = match success?.recv_async().await {
        Ok(Ok(())) => format!("Joined <#{}>!", channel_id),
        Ok(Err(e)) => format!("Failed to join <#{}>! Why: {:?}", channel_id, e),
        _ => format!("Failed to join <#{}>: Gateway error!", channel_id),
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(content)?
        .await?;

    Ok(())
}

async fn leave(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    tracing::debug!(
        "leave command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();

    state
        .songbird
        .leave(guild_id)
        .await?;

    state
        .http
        .create_message(msg.channel_id)
        .content("Left the channel")?
        .await?;

    Ok(())
}

async fn play(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    tracing::debug!(
        "play command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the URL of the audio to play?")?
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;

    let guild_id = msg.guild_id.unwrap();

    if let Ok(song) = Restartable::ytdl(msg.content.clone()) {
        let input = Input::from(song);

        let content = format!(
            "Playing **{:?}** by **{:?}**",
            input.metadata.title.as_ref().unwrap_or(&"<UNKNOWN>".to_string()),
            input.metadata.artist.as_ref().unwrap_or(&"<UNKNOWN>".to_string()),
        );

        state
            .http
            .create_message(msg.channel_id)
            .content(content)?
            .await?;

        if let Some(call_lock) = state.songbird.get(guild_id) {
            let mut call = call_lock.lock().await;
            let handle = call.play_source(input);

            let mut store = state.trackdata.write().await;
            store.insert(guild_id, handle);
        }
    } else {
        state
            .http
            .create_message(msg.channel_id)
            .content("Didn't find any results")?
            .await?;
    }

    Ok(())
}

async fn pause(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    tracing::debug!(
        "pause command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();

    let store = state.trackdata.read().await;
    
    let content = if let Some(handle) = store.get(&guild_id) {
        let info = handle.get_info()?
            .await?;

        let paused = match info.playing {
            PlayMode::Play => {
                let _success = handle.pause();
                false
            }
            _ => {
                let _success = handle.play();
                true   
            }
        };

        let action = if paused { "Unpaused" } else { "Paused" };

        format!("{} the track", action)
    } else {
        format!("No track to (un)pause!")
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(content)?
        .await?;

    Ok(())
}

async fn seek(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    tracing::debug!(
        "seek command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("Where in the track do you want to seek to (in seconds)?")?
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let position = msg.content.parse::<u64>()?;

    let store = state.trackdata.read().await;
    
    let content = if let Some(handle) = store.get(&guild_id) {
        if handle.is_seekable() {
            let _success = handle.seek_time(std::time::Duration::from_secs(position));
            format!("Seeked to {}s", position)
        } else {
            format!("Track is not compatible with seeking!")
        }
    } else {
        format!("No track to seek over!")
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(content)?
        .await?;

    Ok(())
}

async fn stop(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    tracing::debug!(
        "stop command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();

    if let Some(call_lock) = state.songbird.get(guild_id) {
        let mut call = call_lock.lock().await;
        let _ = call.stop();
    }

    state
        .http
        .create_message(msg.channel_id)
        .content("Stopped the track")?
        .await?;

    Ok(())
}

async fn volume(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    tracing::debug!(
        "volume command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the volume you want to set (0.0-10.0, 1.0 being the default)?")?
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let volume = msg.content.parse::<f64>()?;

    if !volume.is_finite() || volume > 10.0 || volume < 0.0 {
        state
            .http
            .create_message(msg.channel_id)
            .content("Invalid volume!")?
            .await?;

        return Ok(());
    }

    let store = state.trackdata.read().await;
    
    let content = if let Some(handle) = store.get(&guild_id) {
        let _success = handle.set_volume(volume as f32);
        format!("Set the volume to {}", volume)
    } else {
        format!("No track to change volume!")
    };

    state
        .http
        .create_message(msg.channel_id)
        .content(content)?
        .await?;

    Ok(())
}
