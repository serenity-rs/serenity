//! In this example, you will be shown how to share data between events.

use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serenity::async_trait;
use serenity::model::event::FullEvent;
use serenity::prelude::*;

// A container type is created for inserting into the Client's `data`, which allows for data to be
// accessible across all events or anywhere else that has a copy of the `data` Arc. These are places
// where either Context or Client is usually present.
struct UserData {
    message_count: AtomicUsize,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                // Since the data is located in Context, you are able to use it within events!
                let data = ctx.data::<UserData>();

                // We are checking if the bot id is the same as the message author id.
                let owo_count = if new_message.author.id != ctx.cache.current_user().id
                    && new_message.content.to_lowercase().contains("owo")
                {
                    // Here, we are checking how many "owo" there are in the message content.
                    let owo_in_msg =
                        new_message.content.to_ascii_lowercase().matches("owo").count();

                    // Atomic operations with ordering do not require `mut` to be modified. Here,
                    // we want to increase the message count by 1.
                    // See: https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_add
                    data.message_count.fetch_add(owo_in_msg, Ordering::SeqCst) + 1
                } else {
                    // We don't need to check for "owo_count" if "owo" isn't in the message!
                    return;
                };

                if new_message.content.starts_with("~owo_count") {
                    let response = if owo_count == 1 {
                        Cow::Borrowed(
                            "You are the first one to say owo this session! *because it's in the command name* :P",
                        )
                    } else {
                        Cow::Owned(format!("OWO has been said {owo_count} times!"))
                    };

                    if let Err(err) = new_message.reply(&ctx.http, response).await {
                        eprintln!("Error sending response: {err:?}")
                    };
                }
            },
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                println!("{} is connected!", data_about_bot.user.name);
            },
            _ => {},
        }
    }
}

#[tokio::main]
async fn main() {
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");

    // We set the initial value for our user data, which we will use throughout the rest of our
    // program.
    let data = UserData {
        message_count: AtomicUsize::new(0),
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        // Specifying the data type as a type argument here is optional, but if done, it guarantees
        // that Context::data will not panic if the same type is given, as providing the incorrect
        // type will lead to a compiler error, rather than a runtime panic.
        .data::<UserData>(Arc::new(data))
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
