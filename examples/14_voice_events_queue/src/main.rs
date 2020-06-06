//! Example demonstrating how to make use of individual track audio events,
//! and how to use the `TrackQueue` system.
//!
//! Requires the "cache", "methods", and "voice" features be enabled in your
//! Cargo.toml, like so:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["cache", "framework", "standard_framework", "voice"]
//! ```
use std::{collections::HashMap, env, time::Duration, sync::Arc};

// Import the client's bridge to the voice manager. Since voice is a standalone
// feature, it's not as ergonomic to work with as it could be. The client
// provides a clean bridged integration with voice.
use serenity::client::bridge::voice::ClientVoiceManager;

// Import the `Context` from the client and `parking_lot`'s `Mutex`.
//
// `parking_lot` offers much more efficient implementations of `std::sync`'s
// types. You can read more about it here:
//
// <https://github.com/Amanieu/parking_lot#features>
use serenity::{client::Context, prelude::Mutex};

use serenity::{
    client::{Client, EventHandler},
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group},
        },
    },
    model::{channel::Message, gateway::Ready, id::GuildId, misc::Mentionable},
    Result as SerenityResult,
    voice::{
        self,
        input::{
            self,
            RestartableSource,
        },
        tracks::TrackQueue,
        Event,
        EventContext,
        TrackEvent,
    },
};

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct VoiceQueueManager;

impl TypeMapKey for VoiceQueueManager {
    type Value = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(deafen, join, leave, mute, play_fade, queue, skip, stop, ping, undeafen, unmute)]
struct General;

fn main() {
    env_logger::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // Obtain a lock to the data owned by the client, and insert the client's
    // voice manager into it. This allows the voice manager to be accessible by
    // event handlers and framework commands.
    {
        let mut data = client.data.write();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<VoiceQueueManager>(Arc::new(Mutex::new(HashMap::new())));
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("~"))
        .group(&GENERAL_GROUP));

    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}

#[command]
fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Groups and DMs not supported"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel"));

            return Ok(());
        },
    };

    if handler.self_deaf {
        check_msg(msg.channel_id.say(&ctx.http, "Already deafened"));
    } else {
        handler.deafen(true);

        check_msg(msg.channel_id.say(&ctx.http, "Deafened"));
    }

    Ok(())
}

#[command]
fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Groups and DMs not supported"));

            return Ok(());
        }
    };

    let guild_id = guild.read().id;

    let channel_id = guild
        .read()
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);


    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel"));

            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handle) = manager.join(guild_id, connect_to) {
        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())));

        let chan_id = msg.channel_id;

        let send_http = ctx.http.clone();

        handle.add_global_event(
            Event::Track(TrackEvent::End),
            move |ctx| {
                if let EventContext::Track(track_list) = ctx {
                    check_msg(chan_id.say(&send_http, &format!("Tracks ended: {}.", track_list.len())));
                }

                None
            },
        );

        let send_http = ctx.http.clone();
        let mut i = 0;

        handle.add_global_event(
            Event::Periodic(Duration::from_secs(60), None),
            move |_ctx| {
                i += 1;
                check_msg(chan_id.say(&send_http, &format!("I've been in this channel for {} minutes!", i)));

                None
            }
        );
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel"));
    }

    Ok(())
}

#[command]
fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Groups and DMs not supported"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel"));
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel"));
    }

    Ok(())
}

#[command]
fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Groups and DMs not supported"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel"));

            return Ok(());
        },
    };

    if handler.self_mute {
        check_msg(msg.channel_id.say(&ctx.http, "Already muted"));
    } else {
        handler.mute(true);

        check_msg(msg.channel_id.say(&ctx.http, "Now muted"));
    }

    Ok(())
}

#[command]
fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong!"));

    Ok(())
}

#[command]
fn play_fade(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio"));

            return Ok(());
        },
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "Must provide a valid URL"));

        return Ok(());
    }

    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match input::ytdl(&url) {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg"));

                return Ok(());
            },
        };

        // This handler object will allow you to, as needed,
        // control the audio track via events and further commands.
        let song = handler.play_source(source.into());
        let send_http = ctx.http.clone();
        let chan_id = msg.channel_id;

        // This shows how to periodically fire an event, in this case to
        // periodically make a track quieter until it can be no longer heard.
        let _ = song.add_event(
            Event::Periodic(Duration::from_secs(5), Some(Duration::from_secs(7))),
            move |evt_ctx| {
                if let EventContext::Track(&[(state, track)]) = evt_ctx {
                    let _ = track.set_volume(state.volume / 2.0);

                    if state.volume < 1e-2 {
                        let _ = track.stop();
                        check_msg(chan_id.say(&send_http, "Stopping song..."));
                        Some(Event::Cancel)
                    } else {
                        check_msg(chan_id.say(&send_http, "Volume reduced."));
                        None
                    }
                } else {
                    None
                }
            },
        );

        let send_http = ctx.http.clone();
        
        // This shows how to fire an event once an audio track completes,
        // either due to hitting the end of the bytestream or stopped by user code.
        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            move |_evt_ctx| {
                check_msg(chan_id.say(&send_http, "Song faded out completely!"));

                None
            },
        );

        check_msg(msg.channel_id.say(&ctx.http, "Playing song"));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in"));
    }

    Ok(())
}

#[command]
fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio"));

            return Ok(());
        },
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "Must provide a valid URL"));

        return Ok(());
    }

    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let queues_lock = ctx.data.read().get::<VoiceQueueManager>().cloned().expect("Expected VoiceQueueManager in ShareMap.");
    let mut manager = manager_lock.lock();
    let mut track_queues = queues_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match input::ytdl(&url) {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg"));

                return Ok(());
            },
        };

        // We need to ensure that this guild has a TrackQueue created for it.
        let queue = track_queues.entry(guild_id)
            .or_default();

        // Queueing a track is this easy!
        queue.add_source(source, handler);

        check_msg(msg.channel_id.say(&ctx.http, format!("Added song to queue: position {}", queue.len())));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in"));
    }

    Ok(())
}

#[command]
fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        },
    };

    let queues_lock = ctx.data.read().get::<VoiceQueueManager>().cloned().expect("Expected VoiceQueueManager in ShareMap.");
    let mut track_queues = queues_lock.lock();

    if let Some(queue) = track_queues.get_mut(&guild_id) {
        let _ = queue.skip();

        check_msg(msg.channel_id.say(&ctx.http, format!("Song skipped: {} in queue.", queue.len())));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in"));
    }

    Ok(())
}

#[command]
fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        },
    };

    let queues_lock = ctx.data.read().get::<VoiceQueueManager>().cloned().expect("Expected VoiceQueueManager in ShareMap.");
    let mut track_queues = queues_lock.lock();

    if let Some(queue) = track_queues.get_mut(&guild_id) {
        let _ = queue.stop();

        check_msg(msg.channel_id.say(&ctx.http, "Queue cleared."));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in"));
    }

    Ok(())
}

#[command]
fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.deafen(false);

        check_msg(msg.channel_id.say(&ctx.http, "Undeafened"));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to undeafen in"));
    }

    Ok(())
}

#[command]
fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        },
    };
    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.mute(false);

        check_msg(msg.channel_id.say(&ctx.http, "Unmuted"));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to unmute in"));
    }

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
