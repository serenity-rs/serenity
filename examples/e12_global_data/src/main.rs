//! In this example, you will be shown various ways of sharing data between events and commands.
//! And how to use locks correctly to avoid deadlocking the bot.
#![allow(deprecated)] // We recommend migrating to poise, instead of using the standard command framework.

use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{Args, CommandResult, Configuration, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

// A container type is created for inserting into the Client's `data`, which allows for data to be
// accessible across all events and framework commands, or anywhere else that has a copy of the
// `data` Arc. These places are usually where either Context or Client is present.
struct UserData {
    message_count: AtomicUsize,
    command_counter: DashMap<String, u64>,
}

#[group]
#[commands(ping, command_usage, owo_count)]
struct General;

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Running command '{}' invoked by '{}'", command_name, msg.author.tag());

    // We want to keep write locks open the least time possible, so we wrap them on a block so they
    // get automatically closed at the end.
    {
        // We have to provide the Data type each time we access Context::data.
        let counter = &ctx.data::<UserData>().command_counter;

        // The DashMap provides interior mutability, meaning we can write to it with a & reference.
        let mut entry = counter.entry(command_name.to_string()).or_insert(0);

        // And we write the amount of times the command has been called to it.
        *entry += 1;
    }

    true
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: &Context, msg: &Message) {
        // We are verifying if the bot id is the same as the message author id.
        if msg.author.id != ctx.cache.current_user().id
            && msg.content.to_lowercase().contains("owo")
        {
            // Since data is located in Context, this means you are able to use it within events!
            let count = &ctx.data::<UserData>().message_count;

            // Here, we are checking how many "owo" there are in the message content.
            let owo_in_msg = msg.content.to_ascii_lowercase().matches("owo").count();

            // Atomic operations with ordering do not require mut to be modified.
            // In this case, we want to increase the message count by 1.
            // https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_add
            count.fetch_add(owo_in_msg, Ordering::SeqCst);
        }
    }

    async fn ready(&self, _: &Context, ready: &Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // We setup the initial value for our user data, which we will use throughout the rest of our
    // program.
    let data = UserData {
        message_count: AtomicUsize::new(0),
        command_counter: DashMap::new(),
    };

    let framework = StandardFramework::new().before(before).group(&GENERAL_GROUP);
    framework.configure(Configuration::new().with_whitespace(true).prefix("~"));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        // We have to use `as` to turn our UserData into `dyn Any`.
        .data(Arc::new(data) as _)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

/// Usage: `~command_usage <command_name>`
/// Example: `~command_usage ping`
#[command]
async fn command_usage(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "I require an argument to run this command.").await?;
            return Ok(());
        },
    };

    // Yet again, we want to keep the locks open for the least time possible.
    let amount = {
        // We again, have to provide the UserData type.
        let data = ctx.data::<UserData>();

        // We fetch the value our of the map, temporarily holding a dashmap read lock.
        data.command_counter.get(&command_name).map_or(0, |x| *x)
    };

    if amount == 0 {
        msg.reply(ctx, format!("The command `{command_name}` has not yet been used.")).await?;
    } else {
        msg.reply(
            ctx,
            format!("The command `{command_name}` has been used {amount} time/s this session!"),
        )
        .await?;
    }

    Ok(())
}

#[command]
async fn owo_count(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data::<UserData>();
    let count = data.message_count.load(Ordering::Relaxed);

    if count == 1 {
        msg.reply(
            ctx,
            "You are the first one to say owo this session! *because it's on the command name* :P",
        )
        .await?;
    } else {
        msg.reply(ctx, format!("OWO Has been said {count} times!")).await?;
    }

    Ok(())
}
