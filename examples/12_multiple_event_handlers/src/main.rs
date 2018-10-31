extern crate serenity;

use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use std::env;

// This example uses two different EventHandlers for two different events. In a
// real project, these handlers could live in different modules or even crates.
// The first handler will reply to "!ping" messages:
struct MessageHandler;

impl EventHandler for MessageHandler {
    fn message(&self, _: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say("Pong!") {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

// And the second handler will log the `ready` event:
struct ReadyHandler;

impl EventHandler for ReadyHandler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    // To combine the two handlers, make a vector. Since these handlers have
    // different types, they need to be boxed.
    let handlers: Vec<Box<EventHandler + Send + Sync>> = vec![
        Box::new(MessageHandler),
        Box::new(ReadyHandler)
    ];
    // Use this vector of handlers as the event handler. Each event will be
    // forwarded to all handlers in order.
    let mut client = Client::new(&token, handlers).expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
