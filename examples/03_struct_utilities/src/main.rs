//! Requires the 'methods' feature flag be enabled in your project's Cargo.toml.
//!
//! This can be activated by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/zeyla/serenity.git"
//! features = ["methods"]
//! ```

extern crate serenity;

use serenity::Client;
use std::env;

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_bot(&token);

    client.on_message(|_context, message| {
        if message.content == "!messageme" {
            // If the `methods` feature is enabled, then model structs will
            // have a lot of useful methods implemented, to avoid using an
            // often otherwise bulky Context, or even much lower-level `rest`
            // method.
            //
            // In this case, you can direct message a User directly by simply
            // calling a method on its instance, with the content of the
            // message.
            if let Err(why) = message.author.dm("Hello!") {
                println!("Error when direct messaging user: {:?}", why);
            }
        }
    });

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
