//! Example demonstrating how to store and convert audio streams which you
//! either want to reuse between servers, or to seek/loop on. See `join`, and `ting`.
//!
//! Requires the "cache", "standard_framework", and "voice" features be enabled in your
//! Cargo.toml, like so:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["cache", "framework", "standard_framework", "voice"]
//! ```
use std::{collections::HashMap, convert::TryInto, env, sync::Arc};

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group},
        },
    },
    model::{channel::Message, gateway::Ready, misc::Mentionable},
    prelude::Mutex,
    Result as SerenityResult,
};

use songbird::{
    input::{
        self,
        cached::{Compressed, Memory},
        Input,
    },
    Bitrate,
    Call,
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    SerenityInit,
    TrackEvent,
};

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

enum CachedSound {
    Compressed(Compressed),
    Uncompressed(Memory),
}

impl From<&CachedSound> for Input {
    fn from(obj: &CachedSound) -> Self {
        use CachedSound::*;
        match obj {
            Compressed(c) => c.new_handle()
                .into(),
            Uncompressed(u) => u.new_handle()
                .try_into()
                .expect("Failed to create decoder for Memory source."),
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c
                   .prefix("~"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    // Obtain a lock to the data owned by the client, and insert the client's
    // voice manager into it. This allows the voice manager to be accessible by
    // event handlers and framework commands.
    {
        let mut data = client.data.write().await;

        // Loading the audio ahead of time.
        let mut audio_map = HashMap::new();

        // Creation of an in-memory source.
        //
        // This is a small sound effect, so storing the whole thing is relatively cheap.
        //
        // `spawn_loader` creates a new thread which works to copy all the audio into memory 
        // ahead of time. We do this in both cases to ensure optimal performance for the audio
        // core.
        let ting_src = Memory::new(
            input::ffmpeg("ting.wav").await.expect("File should be in root folder."),
        ).expect("These parameters are well-defined.");
        let _ = ting_src.raw.spawn_loader();
        audio_map.insert("ting".into(), CachedSound::Uncompressed(ting_src));

        // Another short sting, to show where each loop occurs.
        let loop_src = Memory::new(
            input::ffmpeg("loop.wav").await.expect("File should be in root folder."),
        ).expect("These parameters are well-defined.");
        let _ = loop_src.raw.spawn_loader();
        audio_map.insert("loop".into(), CachedSound::Uncompressed(loop_src));

        // Creation of a compressed source.
        //
        // This is a full song, making this a much less memory-heavy choice.
        //
        // Music by Cloudkicker, used under CC BY-SC-SA 3.0 (https://creativecommons.org/licenses/by-nc-sa/3.0/).
        let song_src = Compressed::new(
                input::ffmpeg("Cloudkicker_-_Loops_-_22_2011_07.mp3").await.expect("Link may be dead."),
                Bitrate::BitsPerSecond(128_000),
            ).expect("These parameters are well-defined.");
        let _ = song_src.raw.spawn_loader();
        audio_map.insert("song".into(), CachedSound::Compressed(song_src));

        data.insert::<SoundStore>(Arc::new(Mutex::new(audio_map)));
    }

    let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
}

#[command]
#[only_in(guilds)]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        check_msg(msg.channel_id.say(&ctx.http, "Already deafened").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Deafened").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);


    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let (handler_lock, success_reader) = manager.join(guild_id, connect_to).await;

    let call_lock_for_evt = handler_lock.clone();

    if let Ok(_reader) = success_reader {
        let mut handler = handler_lock.lock().await;
        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);

        let sources_lock = ctx.data.read().await.get::<SoundStore>().cloned().expect("Sound cache was installed at startup.");
        let sources_lock_for_evt = sources_lock.clone();
        let sources = sources_lock.lock().await;
        let source = sources.get("song").expect("Handle placed into cache at startup.");

        let song = handler.play_source(source.into());
        let _ = song.set_volume(1.0);
        let _ = song.enable_loop();

        // Play a guitar chord whenever the main backing track loops.
        let _ = song.add_event(
            Event::Track(TrackEvent::Loop),
            LoopPlaySound {
                call_lock: call_lock_for_evt,
                sources: sources_lock_for_evt,
            },
        );
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
    }

    Ok(())
}

struct LoopPlaySound {
    call_lock: Arc<Mutex<Call>>,
    sources: Arc<Mutex<HashMap<String, CachedSound>>>,
}

#[async_trait]
impl VoiceEventHandler for LoopPlaySound {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let src = {
            let sources = self.sources.lock().await;
            sources.get("loop").expect("Handle placed into cache at startup.").into()
        };

        let mut handler = self.call_lock.lock().await;
        let sound = handler.play_source(src);
        let _ = sound.set_volume(0.5);

        None
    }
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        check_msg(msg.channel_id.say(&ctx.http, "Already muted").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Now muted").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn ting(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let sources_lock = ctx.data.read().await.get::<SoundStore>().cloned().expect("Sound cache was installed at startup.");
        let sources = sources_lock.lock().await;
        let source = sources.get("ting").expect("Handle placed into cache at startup.");

        let _sound = handler.play_source(source.into());

        check_msg(msg.channel_id.say(&ctx.http, "Ting!").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        if let Err(e) = handler.deafen(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Undeafened").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to undeafen in").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        if let Err(e) = handler.mute(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Unmuted").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to unmute in").await);
    }

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
