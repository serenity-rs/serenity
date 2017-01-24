extern crate serenity;

use serenity::Client;
use std::env;

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    // Logging in is essentially equivalent to logging in as a user.
    //
    // The primary difference is that by using `login_user`, the "Bot " string
    // is not prefixed to the token.
    //
    // Additionally, the Client will now know that you are a user, and will
    // disallow you from performing bot-only commands.
    let mut client = Client::login_user(&token);

    client.on_ready(|_ctx, ready| {
        println!("{} is connected!", ready.user.name);
    });

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
