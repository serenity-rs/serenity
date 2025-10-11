use std::sync::Arc;

use serenity::async_trait;
use serenity::prelude::*;

struct Handler;

use serenity::gateway::client::FullEvent;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, _: &Context, event: &FullEvent) {
        match event {
            // This event will be dispatched for guilds, but not for direct messages.
            FullEvent::Message {
                new_message, ..
            } => println!("Received message: {}", new_message.content),
            // As the intents set in this example, this event shall never be dispatched.
            // Try it by changing your status.
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
    // Configure the client with your Discord bot token in the environment.
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");

    // Intents are a bitflag, bitwise operations can be used to dictate which intents to use
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    // Build our client.
    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(Handler))
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
