//! This example showcases a basic ping bot.
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::event::FullEvent;
use serenity::prelude::*;

// Define the event handler struct.
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Events are dispatched via the `dispatch()` method, using a thread pool so that multiple
    // events can be dispatched simultaneously.
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            // Set a handler for the `Message` event. This is called whenever a new message is
            // received.
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content == "!ping" {
                    // Sending a message can fail due to a network error, authentication error, or
                    // lack of permissions to post in the channel, so log to stdout when some error
                    // occurs, with a description of the error.
                    if let Err(why) = new_message.channel_id.say(&ctx.http, "Pong!").await {
                        println!("Error sending message: {why:?}");
                    }
                }
            },

            // Set a handler to be called on the `Ready` event. This is called when a shard
            // successfully connects and a READY payload is sent by Discord. This payload contains
            // data like the current user's user data, guilds, and more.
            //
            // Here, we just print the current user's username.
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
    // Set gateway intents to determine which events the bot will receive.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    // Finally, start a single shard and begin listening to events.
    //
    // Shards will automatically attempt to reconnect, performing exponential backoff until
    // successful.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
