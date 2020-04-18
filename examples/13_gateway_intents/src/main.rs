use std::env;

use serenity::{
    client::bridge::gateway::GatewayIntents,
    model::{channel::Message, event::PresenceUpdateEvent, gateway::Ready},
    prelude::*,
};

struct Handler;

impl EventHandler for Handler {
    // This event will be dispatched for guilds, but not for direct messages.
    fn message(&self, _ctx: Context, msg: Message) {
        println!("Received message: {}", msg.content);
    }

    // As the intents set in this example, this event shall never be dispatched.
    // Try it by changing your status.
    fn presence_update(&self, _ctx: Context, _new_data: PresenceUpdateEvent) {
        println!("Presence Update");
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a client with extras and then specify the intents you want to
    // use.
    // By default, Serenity sets no intents.
    let mut client = Client::new_with_extras(&token, |f| {
        f.intents(GatewayIntents::GUILDS).event_handler(Handler)
    })
    .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
