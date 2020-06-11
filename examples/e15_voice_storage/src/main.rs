//! Example demonstrating how to store and convert audio streams which you
//! either want to reuse between servers, or to seek/loop on. See `join`, and `ting`.
//!
//! Requires the "cache", "methods", and "voice" features be enabled in your
//! Cargo.toml, like so:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["cache", "framework", "standard_framework", "voice"]
//! ```
use std::{collections::HashMap, env, sync::Arc};

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
    model::{channel::Message, gateway::Ready, misc::Mentionable},
    Result as SerenityResult,
    voice::{
        self,
        Bitrate,
        Event,
        input::{
            self,
            cached::{
                CompressedSource,
                CompressedSourceBase,
                MemorySource,
            },
            Input,
        },
        TrackEvent,
    },
};

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

enum CachedSound {
    Compressed(CompressedSourceBase),
    Uncompressed(MemorySource),
}

impl From<&CachedSound> for Input {
    fn from(obj: &CachedSound) -> Self {
        use CachedSound::*;
        match obj {
            Compressed(c) => c.new_handle()
                .expect("Opus errors on decoder creation are rare if we're copying valid settings.")
                .into(),
            Uncompressed(u) => u.new_handle().into(),
        }
    }
}

struct SoundStore;

impl TypeMapKey for SoundStore {
    type Value = Arc<Mutex<HashMap<String, CachedSound>>>;
}

#[group]
#[commands(deafen, join, leave, mute, ting, undeafen, unmute)]
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

        // Loading the audio ahead of time.
        let mut audio_map = HashMap::new();

        // Creation of an in-memory source.
        //
        // This is a small sound effect, so storing the whole thing is relatively cheap.
        //
        // `spawn_loader` creates a new thread which works to copy all the audio into memory 
        // ahead of time. We do this in both cases to ensure optimal performance for the audio
        // core.
        let ting_src = MemorySource::new(
            input::ffmpeg("ting.wav").expect("File should be in root folder."),
            None);
        let _ = ting_src.spawn_loader();
        audio_map.insert("ting".into(), CachedSound::Uncompressed(ting_src));

        // Another short sting, to show where each loop occurs.
        let loop_src = MemorySource::new(
            input::ffmpeg("loop.wav").expect("File should be in root folder."),
            None);
        let _ = loop_src.spawn_loader();
        audio_map.insert("loop".into(), CachedSound::Uncompressed(loop_src));

        // Creation of a compressed source.
        //
        // This is a full song, making this a much less memory-heavy choice.
        //
        // Music by Cloudkicker, used under CC BY 3.0 (https://creativecommons.org/licenses/by/3.0/).
        let song_src = CompressedSource::new(
                input::ytdl("https://cloudkicker.bandcamp.com/track/7").expect("Link may be dead."),
                Bitrate::BitsPerSecond(128_000),
                None,
            ).expect("These parameters are well-defined.");
        let _ = song_src.spawn_loader();
        // CompressedSource cannot be sent between threads, so we need to discard some state using
        // `into_sendable`.
        audio_map.insert("song".into(), CachedSound::Compressed(song_src.into_sendable()));

        data.insert::<SoundStore>(Arc::new(Mutex::new(audio_map)));
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
    let manager_lock_for_evt = manager_lock.clone();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.join(guild_id, connect_to) {
        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())));

        let sources_lock = ctx.data.read().get::<SoundStore>().cloned().expect("Sound cache was installed at startup.");
        let sources_lock_for_evt = sources_lock.clone();
        let sources = sources_lock.lock();
        let source = sources.get("song").expect("Handle placed into cache at startup.");

        let song = handler.play_source(source.into());
        let _ = song.set_volume(0.5);
        let _ = song.enable_loop();

        // Play a guitar chord whenever the main backing track loops.
        let _ = song.add_event(
            Event::Track(TrackEvent::Loop),
            move |_evt_ctx| {
                let src = {
                    let sources = sources_lock_for_evt.lock();
                    sources.get("loop").expect("Handle placed into cache at startup.").into()
                };

                let mut manager = manager_lock_for_evt.lock();
                if let Some(handler) = manager.get_mut(guild_id) {
                    let sound = handler.play_source(src);
                    let _ = sound.set_volume(0.5);
                }

                None
            },
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
fn ting(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
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
        let sources_lock = ctx.data.read().get::<SoundStore>().cloned().expect("Sound cache was installed at startup.");
        let sources = sources_lock.lock();
        let source = sources.get("ting").expect("Handle placed into cache at startup.");

        let _sound = handler.play_source(source.into());

        check_msg(msg.channel_id.say(&ctx.http, "Ting!"));
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
