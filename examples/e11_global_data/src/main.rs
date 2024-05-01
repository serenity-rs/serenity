//! In this example, you will be shown how to share data between events.

use std::borrow::Cow;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

// A container type is created for inserting into the Client's `data`, which allows for data to be
// accessible across all events or anywhere else that has a copy of the `data` Arc.
// These places are usually where either Context or Client is present.
struct UserData {
    message_count: AtomicUsize,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Since data is located in Context, this means you are able to use it within events!
        let data = ctx.data::<UserData>();

        // We are verifying if the bot id is the same as the message author id.
        let owo_count = if msg.author.id != ctx.cache.current_user().id
            && msg.content.to_lowercase().contains("owo")
        {
            // Here, we are checking how many "owo" there are in the message content.
            let owo_in_msg = msg.content.to_ascii_lowercase().matches("owo").count();

            // Atomic operations with ordering do not require mut to be modified.
            // In this case, we want to increase the message count by 1.
            // https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_add
            data.message_count.fetch_add(owo_in_msg, Ordering::SeqCst) + 1
        } else {
            // We don't need to check for "owo_count" if "owo" isn't in the message!
            return;
        };

        if msg.content.starts_with("~owo_count") {
            let response = if owo_count == 1 {
                Cow::Borrowed("You are the first one to say owo this session! *because it's on the command name* :P")
            } else {
                Cow::Owned(format!("OWO Has been said {owo_count} times!"))
            };

            if let Err(err) = msg.reply(&ctx.http, response).await {
                eprintln!("Error sending response: {err:?}")
            };
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
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
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        // Specifying the data type as a type argument here is optional, but if done, you can
        // guarantee that Context::data will not panic if the same type is given, as providing the
        // incorrect type will lead to a compiler error, rather than a runtime panic.
        .data::<UserData>(Arc::new(data))
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
