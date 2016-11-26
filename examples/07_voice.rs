// Requires the feature "voice" be enabled.

extern crate serenity;

#[cfg(feature = "voice")]
use serenity::client::{CACHE, Client, Context};
#[cfg(feature = "voice")]
use serenity::model::{Channel, ChannelId, Message};
#[cfg(feature = "voice")]
use std::env;

#[cfg(not(feature = "voice"))]
fn main() {
    panic!("Voice not enabled");
}

#[cfg(feature = "voice")]
fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_bot(&token);

    client.with_framework(|f|
        f.configure(|c| c.prefix("~"))
        .on("deafen", deafen)
        .on("join", join)
        .on("leave", leave)
        .on("mute", mute)
        .on("ping", ping));

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    let _ = client.start();
}

#[cfg(feature = "voice")]
fn deafen(context: Context, message: Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_channel(message.channel_id) {
        Some(Channel::Guild(channel)) => channel.guild_id,
        Some(_) => {
            let _ = message.reply("Groups and DMs not supported");

            return;
        },
        None => {
            let _ = context.say("Can't find guild");

            return;
        },
    };

    let mut shard = context.shard.lock().unwrap();

    let handler = match shard.manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            let _ = message.reply("Not in a voice channel");

            return;
        },
    };

    if handler.is_deafened() {
        let _ = context.say("Already deafened");
    } else {
        handler.deafen(true);

        let _ = context.say("Deafened");
    }
}

#[cfg(feature = "voice")]
fn join(context: Context, message: Message, args: Vec<String>) {
    let connect_to = match args.get(0) {
        Some(arg) => match arg.parse::<u64>() {
            Ok(id) => ChannelId(id),
            Err(_why) => {
                let _ = message.reply("Invalid voice channel ID given");

                return;
            },
        },
        None => {
            let _ = message.reply("Requires a voice channel ID be given");

            return;
        },
    };

    let guild_id = match CACHE.read().unwrap().get_channel(message.channel_id) {
        Some(Channel::Guild(channel)) => channel.guild_id,
        Some(_) => {
            let _ = context.say("Groups and DMs not supported");

            return;
        },
        None => {
            let _ = context.say("Can't find guild");

            return;
        },
    };

    let mut shard = context.shard.lock().unwrap();
    let mut manager = &mut shard.manager;

    let _handler = manager.join(Some(guild_id), connect_to);

    let _ = context.say(&format!("Joined {}", connect_to.mention()));
}

#[cfg(feature = "voice")]
fn leave(context: Context, message: Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_channel(message.channel_id) {
        Some(Channel::Guild(channel)) => channel.guild_id,
        Some(_) => {
            let _ = context.say("Groups and DMs not supported");

            return;
        },
        None => {
            let _ = context.say("Can't find guild");

            return;
        },
    };

    let is_connected = match context.shard.lock().unwrap().manager.get(guild_id) {
        Some(handler) => handler.channel().is_some(),
        None => false,
    };

    if is_connected {
        context.shard.lock().unwrap().manager.remove(guild_id);

        let _ = context.say("Left voice channel");
    } else {
        let _ = message.reply("Not in a voice channel");
    }
}

#[cfg(feature = "voice")]
fn mute(context: Context, message: Message, _args: Vec<String>) {
    let guild_id = match CACHE.read().unwrap().get_channel(message.channel_id) {
        Some(Channel::Guild(channel)) => channel.guild_id,
        Some(_) => {
            let _ = message.reply("Groups and DMs not supported");

            return;
        },
        None => {
            let _ = context.say("Can't find guild");

            return;
        },
    };

    let mut shard = context.shard.lock().unwrap();

    let handler = match shard.manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            let _ = message.reply("Not in a voice channel");

            return;
        },
    };

    if handler.is_muted() {
        let _ = context.say("Already muted");
    } else {
        handler.mute(true);

        let _ = context.say("Now muted");
    }
}

#[cfg(feature = "voice")]
fn ping(context: Context, _message: Message, _args: Vec<String>) {
    let _ = context.say("Pong!");
}
