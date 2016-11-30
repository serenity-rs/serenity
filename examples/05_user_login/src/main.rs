extern crate serenity;

use serenity::Client;
use std::env;

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_user(&token);

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    println!("{:?}", client.start());
}
