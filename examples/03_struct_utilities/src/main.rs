extern crate serenity;

use serenity::Client;
use std::env;

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login(&token);

    client.on_message(|_ctx, msg| {
        if msg.content == "!messageme" {
            // If the `methods` feature is enabled, then model structs will
            // have a lot of useful methods implemented, to avoid using an
            // often otherwise bulky Context, or even much lower-level `rest`
            // method.
            //
            // In this case, you can direct message a User directly by simply
            // calling a method on its instance, with the content of the
            // message.
            if let Err(why) = msg.author.dm("Hello!") {
                println!("Error when direct messaging user: {:?}", why);
            }
        }
    });

    client.on_ready(|_ctx, ready| {
        println!("{} is connected!", ready.user.name);
    });

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
