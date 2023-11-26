//! In this example, you will be shown various ways of sharing data between events and commands.
//! And how to use locks correctly to avoid deadlocking the bot.

use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{Args, CommandResult, Configuration, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

// A container type is created for inserting into the Client's `data`, which allows for data to be
// accessible across all events and framework commands, or anywhere else that has a copy of the
// `data` Arc. These places are usually where either Context or Client is present.
#[derive(Default)]
struct Data {
    // We use the `dashmap` crate, used inside serenity for caching,
    // to avoid locking and unlocking a HashMap ourselves.
    command_counter: dashmap::DashMap<String, u64>,
    // While you will be using locking mechanisms most of the time you want to modify data,
    // sometimes it's not required; like for example, with static data, or if you are using
    // other kinds of atomic operators.
    message_count: AtomicUsize,
}

type Context = serenity::client::Context<Data>;

#[group]
#[commands(ping, command_usage, owo_count)]
struct General<Data>;

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Running command '{}' invoked by '{}'", command_name, msg.author.tag());

    // We want to keep locks open the least time possible, so we wrap them
    // on a block so they get automatically closed at the end.
    {
        // Since we are using a `dashmap`, we do not need to wrap it in any locks.
        let counter = &ctx.data.command_counter;

        // We open the lock to the specific key, which can never cross an `await`.
        // And we write the amount of times the command has been called to it.
        let mut entry = counter.entry(command_name.to_string()).or_insert(0);
        *entry += 1;
    }

    true
}

struct Handler;

#[async_trait]
impl EventHandler<Data> for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // We are verifying if the bot id is the same as the message author id.
        if msg.author.id != ctx.cache.current_user().id
            && msg.content.to_lowercase().contains("owo")
        {
            // Since data is located in Context, this means you are able to use it within events!
            let count = &ctx.data.message_count;

            // Here, we are checking how many "owo" there are in the message content.
            let owo_in_msg = msg.content.to_ascii_lowercase().matches("owo").count();

            // Atomic operations with ordering do not require mut to be modified.
            // In this case, we want to increase the message count by 1.
            // https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_add
            count.fetch_add(owo_in_msg, Ordering::SeqCst);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new().before(before).group(&GENERAL_GROUP);
    framework.configure(Configuration::new().with_whitespace(true).prefix("~"));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents, Data::default())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}

#[command]
async fn ping<Data>(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

/// Usage: `~command_usage <command_name>`
/// Example: `~command_usage ping`
#[command]
async fn command_usage<Data>(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "I require an argument to run this command.").await?;
            return Ok(());
        },
    };

    // Yet again, we want to keep the locks open for the least time possible.
    let amount = {
        // We grab a reference to our command_counter DashMap, and again don't need to lock it.
        let command_counter = &ctx.data.command_counter;

        // And we return a usable value from it.
        // This time, the value is not Arc, so the data will be cloned.
        command_counter.get(&command_name).map_or(0, |amount| *amount)
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
async fn owo_count<Data>(ctx: &Context, msg: &Message) -> CommandResult {
    let raw_count = &ctx.data.message_count;
    let count = raw_count.load(Ordering::Relaxed);

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
