//! This example demonstrates how to use gateway intents.
//!
//! Gateway intents limit the types of events a bot receives from the gateway. Less events means
//! less data for your bot to process, so it's best to pick only the intents you need.
use std::sync::Arc;

use serenity::async_trait;
use serenity::prelude::*;

struct Handler;

use serenity::model::event::FullEvent;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, _: &Context, event: &FullEvent) {
        match event {
            // This event will be dispatched for guilds, but not for direct messages since we did
            // not include the `DIRECT_MESSAGES` intent.
            FullEvent::Message {
                new_message, ..
            } => {
                println!("Received message: {}", new_message.content)
            },
            // Due to the intents set in this example, this event will never be dispatched. You can
            // confirm this by changing your status and watching for the "Presence Update" text in
            // the console.
            FullEvent::PresenceUpdate {
                ..
            } => {
                println!("Presence Update")
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

    // Intents are a bitflag; bitwise operations can be used to dictate which intents to use.
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
