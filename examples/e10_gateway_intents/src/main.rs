use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{Presence, Ready};
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // This event will be dispatched for guilds, but not for direct messages.
    async fn message(&self, _ctx: Context, msg: Message) {
        println!("Received message: {}", msg.content);
    }

    // As the intents set in this example, this event shall never be dispatched.
    // Try it by changing your status.
    async fn presence_update(
        &self,
        _ctx: Context,
        _old_data: Option<Presence>,
        _new_data: Presence,
    ) {
        println!("Presence Update");
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
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
        .event_handler(Handler)
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
