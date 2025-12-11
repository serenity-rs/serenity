use std::sync::Arc;

use serenity::async_trait;
use serenity::gateway::client::FullEvent;
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
                    let channel = match new_message.channel(ctx).await {
                        Ok(channel) => channel,
                        Err(why) => {
                            println!("Error getting channel: {why:?}");

                            return;
                        },
                    };

                    // The message builder allows for creating a message by mentioning users
                    // dynamically, pushing "safe" versions of content (such as
                    // bolding normalized content), displaying emojis, and more.
                    let response = MessageBuilder::new()
                        .push("User ")
                        .push_bold_safe(new_message.author.name.as_str())
                        .push(" used the 'ping' command in the ")
                        .mention(&channel)
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
    // Configure the client with your Discord bot token in the environment.
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
