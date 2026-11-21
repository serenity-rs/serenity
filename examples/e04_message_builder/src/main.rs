//! This example demonstrates use of the message builder.
//!
//! The message builder is an ergonomic utility used to build messages safely by pushing "safe"
//! versions of content and dynamically mentioning mentionable structs, such as users, channels,
//! or roles.
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::event::FullEvent;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content == "!ping" {
                    // Here we use the message builder to push normalized bold content and include
                    // a channel mention.
                    let response = MessageBuilder::new()
                        .push("User ")
                        .push_bold_safe(new_message.author.name.as_str())
                        .push(" used the 'ping' command in the ")
                        .channel(new_message.channel_id)
                        .push(" channel")
                        .build();

                    if let Err(why) = new_message.channel_id.say(&ctx.http, &response).await {
                        println!("Error sending message: {why:?}");
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
