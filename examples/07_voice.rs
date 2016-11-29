// Requires the feature "voice" be enabled.

extern crate serenity;

#[cfg(feature = "voice")]
use serenity::client::{CACHE, Client, Context};
#[cfg(feature = "voice")]
use serenity::ext::voice;
#[cfg(feature = "voice")]
use serenity::model::{ChannelId, Message};
#[cfg(feature = "voice")]
use serenity::Result as SerenityResult;
#[cfg(feature = "voice")]
use std::env;

#[cfg(not(feature = "voice"))]
fn main() {
    panic!("'extras' and 'voice' must be enabled");
}

#[cfg(feature = "voice")]
fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_bot(&token);

    client.with_framework(|f| f
        .configure(|c| c
            .prefix("~")
            .on_mention(true))
        .on("deafen", deafen)
        .on("join", join)
        .on("leave", leave)
        .on("mute", mute)
        .on("play", play)
        .on("ping", ping)
        .on("undeafen", undeafen)
        .on("unmute", unmute));

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}

#[cfg(feature = "voice")]
fn deafen(context: &Context, message: &Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Groups and DMs not supported"));

            return;
        },
    };

    let mut shard = context.shard.lock().unwrap();

    let handler = match shard.manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(message.reply("Not in a voice channel"));

            return;
        },
    };

    if handler.is_deafened() {
        check_msg(context.say("Already deafened"));
    } else {
        handler.deafen(true);

        check_msg(context.say("Deafened"));
    }
}

#[cfg(feature = "voice")]
fn join(context: &Context, message: &Message, args: Vec<String>) {
    let connect_to = match args.get(0) {
        Some(arg) => match arg.parse::<u64>() {
            Ok(id) => ChannelId(id),
            Err(_why) => {
                check_msg(message.reply("Invalid voice channel ID given"));

                return;
            },
        },
        None => {
            check_msg(message.reply("Requires a voice channel ID be given"));

            return;
        },
    };

    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Groups and DMs not supported"));

            return;
        },
    };

    let mut shard = context.shard.lock().unwrap();
    shard.manager.join(Some(guild_id), connect_to);

    check_msg(context.say(&format!("Joined {}", connect_to.mention())));
}

#[cfg(feature = "voice")]
fn leave(context: &Context, message: &Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Groups and DMs not supported"));

            return;
        },
    };

    let mut shard = context.shard.lock().unwrap();
    let has_handler = shard.manager.get(guild_id).is_some();

    if has_handler {
        shard.manager.remove(guild_id);

        check_msg(context.say("Left voice channel"));
    } else {
        check_msg(message.reply("Not in a voice channel"));
    }
}

#[cfg(feature = "voice")]
fn mute(context: &Context, message: &Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Groups and DMs not supported"));

            return;
        },
    };

    let mut shard = context.shard.lock().unwrap();

    let handler = match shard.manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(message.reply("Not in a voice channel"));

            return;
        },
    };

    if handler.is_muted() {
        check_msg(context.say("Already muted"));
    } else {
        handler.mute(true);

        check_msg(context.say("Now muted"));
    }
}

#[cfg(feature = "voice")]
fn ping(context: &Context, _message: &Message, _args: Vec<String>) {
    check_msg(context.say("Pong!"));
}

#[cfg(feature = "voice")]
fn play(context: &Context, message: &Message, args: Vec<String>) {
    let url = match args.get(0) {
        Some(url) => url,
        None => {
            check_msg(context.say("Must provide a URL to a video or audio"));

            return;
        },
    };

    if !url.starts_with("http") {
        check_msg(context.say("Must provide a valid URL"));

        return;
    }

    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Error finding channel info"));

            return;
        },
    };

    if let Some(handler) = context.shard.lock().unwrap().manager.get(guild_id) {
        let source = match voice::ytdl(url) {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(context.say("Error sourcing ffmpeg"));

                return;
            },
        };

        handler.play(source);

        check_msg(context.say("Playing song"));
    } else {
        check_msg(context.say("Not in a voice channel to play in"));
    }
}

#[cfg(feature = "voice")]
fn undeafen(context: &Context, message: &Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Error finding channel info"));

            return;
        },
    };

    if let Some(handler) = context.shard.lock().unwrap().manager.get(guild_id) {
        handler.deafen(false);

        check_msg(context.say("Undeafened"));
    } else {
        check_msg(context.say("Not in a voice channel to undeafen in"));
    }
}

#[cfg(feature = "voice")]
fn unmute(context: &Context, message: &Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Error finding channel info"));

            return;
        },
    };

    if let Some(handler) = context.shard.lock().unwrap().manager.get(guild_id) {
        handler.mute(false);

        check_msg(context.say("Unmuted"));
    } else {
        check_msg(context.say("Not in a voice channel to undeafen in"));
    }
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
