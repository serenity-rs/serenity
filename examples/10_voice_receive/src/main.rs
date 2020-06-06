//! Requires the "client", "standard_framework", and "voice" features be enabled
//! in your Cargo.toml, like so:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["client", "standard_framework", "voice"]
//! ```
use std::{env, sync::Arc};

use serenity::{
    client::{bridge::voice::ClientVoiceManager, Client, Context, EventHandler},
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group},
        },
    },
    model::{
        channel::Message,
        event::{VoiceClientConnect, VoiceClientDisconnect, VoiceSpeaking},
        gateway::Ready,
        id::ChannelId,
        misc::Mentionable,
    },
    prelude::*,
    voice::{CoreEvent, EventContext},
    Result as SerenityResult,
};

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

#[group]
#[commands(join, leave, ping)]
struct General;

fn main() {
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
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("~"))
        .group(&GENERAL_GROUP));

    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}

#[command]
fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let connect_to = match args.single::<u64>() {
        Ok(id) => ChannelId(id),
        Err(_) => {
            check_msg(msg.reply(ctx, "Requires a valid voice channel ID be given"));

            return Ok(());
        },
    };

    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "DMs not supported"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.join(guild_id, connect_to) {
        // Receiving audio (and similar tasks) are achieved
        // by creating a global event handler.
        handler.add_global_event(
            CoreEvent::SpeakingStateUpdate.into(),
            |ctx| {
                if let EventContext::SpeakingStateUpdate(
                    VoiceSpeaking {speaking, ssrc, user_id, ..}
                ) = ctx {
                    // You can implement logic here so that you can differentiate users'
                    // SSRCs and map the SSRC to the User ID and maintain this state.
                    // Using this map, you can map the `ssrc` in `voice_packet`
                    // to the user ID and handle their audio packets separately.

                    println!(
                        "Speaking state update: user {:?} has SSRC {:?}, using {:?}",
                        user_id,
                        ssrc,
                        speaking,
                    );
                }

                None
            }
        );

        handler.add_global_event(
            CoreEvent::SpeakingUpdate.into(),
            |ctx| {
                if let EventContext::SpeakingUpdate {ssrc, speaking} = ctx {
                    // You can implement logic here which reacts to a user starting
                    // or stopping speaking.

                    println!(
                        "Source {} has {} speaking.",
                        ssrc,
                        if *speaking {"started"} else {"stopped"},
                    );
                }

                None
            }
        );

        handler.add_global_event(
            CoreEvent::VoicePacket.into(),
            |ctx| {
                if let EventContext::VoicePacket {audio, packet, payload_offset} = ctx {
                    // An event which fires for every received audio packet,
                    // containing the decoded data.

                    println!("Audio packet's first 5 samples: {:?}", audio.get(..5.min(audio.len())));
                    println!(
                        "Audio packet sequence {:05} has {:04} bytes (decompressed from {}), SSRC {}",
                        packet.sequence.0,
                        audio.len() * std::mem::size_of::<i16>(),
                        packet.payload.len(),
                        packet.ssrc,
                    );
                }

                None
            }
        );

        handler.add_global_event(
            CoreEvent::RtcpPacket.into(),
            |ctx| {
                if let EventContext::RtcpPacket {packet, payload_offset} = ctx {
                    // An event which fires for every received rtcp packet,
                    // containing the call statistics and reporting information.
                    println!("RTCP packet received: {:?}", packet);
                }

                None
            }
        );

        handler.add_global_event(
            CoreEvent::ClientConnect.into(),
            |ctx| {
                if let EventContext::ClientConnect(
                    VoiceClientConnect {audio_ssrc, video_ssrc, user_id, ..}
                ) = ctx {
                    // You can implement your own logic here to handle a user who has joined the
                    // voice channel e.g., allocate structures, map their SSRC to User ID.

                    println!(
                        "Client connected: user {:?} has audio SSRC {:?}, video SSRC {:?}",
                        user_id,
                        audio_ssrc,
                        video_ssrc,
                    );
                }

                None
            }
        );

        handler.add_global_event(
            CoreEvent::ClientDisconnect.into(),
            |ctx| {
                if let EventContext::ClientDisconnect(
                    VoiceClientDisconnect {user_id, ..}
                ) = ctx {
                    // You can implement your own logic here to handle a user who has left the
                    // voice channel e.g., finalise processing of statistics etc.
                    // You will typically need to map the User ID to their SSRC; observed when
                    // speaking or connecting.

                    println!("Client disconnected: user {:?}", user_id);
                }

                None
            }
        );

        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())));
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
            check_msg(msg.channel_id.say(&ctx.http, "DMs not supported"));

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);

        check_msg(msg.channel_id.say(&ctx.http,"Left voice channel"));
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel"));
    }

    Ok(())
}

#[command]
fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http,"Pong!"));

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
