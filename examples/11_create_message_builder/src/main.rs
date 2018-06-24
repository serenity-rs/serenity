extern crate serenity;

use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if msg.content == "!hello" {
            // The create message builder allows you to easily create embeds and messages
            // using a builder syntax.
            // This example will create a message that says "Hello, World!", with an embed that has
            // a title, description, three fields, and footer.
            if let Err(why) = msg.channel_id.send_message(|m| m
                .content("Hello, World!")
                .embed(|e| e
                    .title("This is a title")
                    .description("This is a description")
                    .fields(vec![
                        ("This is the first field", "This is a field body", true),
                        ("This is the second field", "Both of these fields are inline", true),
                    ])
                    .field("This is the third field", "This is not an inline field", false)
                    .footer(|f| f
                        .text("This is a footer")))) {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
