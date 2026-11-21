//! If the `model` feature is enabled, many model types will have useful wrapper methods
//! implemented, allowing you to avoid using low-level HTTP requests.
//!
//! This example is a simple demonstration of this functionality.
use std::sync::Arc;

use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::event::FullEvent;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content == "!messageme" {
                    // Here, we send a direct message to a user by simply calling the `dm()` method
                    // on the `UserId` model type.
                    let builder = CreateMessage::new().content("Hello!");
                    let dm = new_message.author.id.dm(&ctx.http, builder).await;

                    if let Err(why) = dm {
                        println!("Error when direct messaging user: {why:?}");
                    }
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
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
