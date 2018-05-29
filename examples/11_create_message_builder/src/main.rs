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
            // a title, description, and footer.
            let msg = msg.channel_id.send_message(|mut m| {
                m.content("Hello, World!");
                m.embed(|mut e| {
                    e.title("This is a title");
                    e.description("This is a description");

                    e.footer(|mut f| {
                        f.text("This is a footer");

                        f
                    });

                    e
                });

                m
            });

            if let Err(why) = msg {
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
