//! Requires the 'methods' feature flag be enabled.

extern crate serenity;

#[cfg(feature = "methods")]
use serenity::Client;
#[cfg(feature = "methods")]
use std::env;

#[cfg(feature = "methods")]
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

#[cfg(not(feature = "methods"))]
fn main() {
    println!("The 'methods' feature flag is required for this example.");
}
