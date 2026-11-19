//! Serenity implements transparent sharding in a way that you do not need to handle separate
//! processes or connections manually.
//!
//! Transparent sharding is useful for a shared cache. Instead of having caches with duplicated
//! data, a shared cache means all your data can be easily accessible across all shards.
//!
//! If your bot is in many guilds, or has surpassed the single-shard maximum of 2500, then you
//! should/must use guild sharding.
//!
//! This example shows how guild sharding works. For the effects to be seen properly, your bot
//! should be in at least two guilds.
//!
//! Assuming a scenario of two guilds: If you say "!ping" in one guild, it should print either
//! "Shard 0" or "Shard 1" in the console. Saying "!ping" in the other guild should print the
//! other number in the console. This confirms that guild sharding works.
use std::sync::Arc;

use serenity::async_trait;
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
                if new_message.content == "!ping" {
                    // Print the shard ID to the console.
                    println!("Shard {}", ctx.shard_id);

                    if let Err(why) = new_message.channel_id.say(&ctx.http, "Pong!").await {
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

    // Specify the total number of shards to use. The individual shard ID is indexed at 0, while
    // the total shard count is indexed at 1.
    //
    // This means if you have 5 shards, your total shard count will be 5, while the individual
    // shards will be assigned numbers 0 through 4.
    if let Err(why) = client.start_shards(2).await {
        println!("Client error: {why:?}");
    }
}
