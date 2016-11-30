//! Requires the 'methods' feature flag be enabled in your project's Cargo.toml.
//!
//! This can be activated by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/zeyla/serenity.rs.git"
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
            let _ = message.author.dm("Hello!");
        }
    });

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    let _ = client.start();
}
