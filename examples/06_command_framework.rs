extern crate serenity;

use serenity::client::Context;
use serenity::Client;
use serenity::model::Message;
use std::env;

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_bot(&token);

    client.on_message(|_context, message| {
        println!("Received message: {:?}", message);
    });

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    // Commands are equivilant to:
    // "~about"
    // "~ping"
    // "~emoji cat"
    // "~emoji dog"
    // "~some complex command"
    client.with_framework(|f| f
        .configure(|c| c
            .on_mention(true)
            .allow_whitespace(true)
            .prefix("~"))
        .on("ping", ping_command)
        .set_check("ping", owner_check) // Ensure only the owner can run this
        .on("emoji cat", cat_command)
        .on("emoji dog", dog_command)
        .on("some complex command", some_complex_command)
        // Commands can be in closure-form as well
        .on("about", |context, _message| drop(context.say("A test bot"))));

    let _ = client.start();
}

fn cat_command(context: Context, _message: Message) {
    let _ = context.say(":cat:");
}

fn dog_command(context: Context, _message: Message) {
    let _ = context.say(":dog:");
}

fn ping_command(_context: Context, message: Message) {
    let _ = message.reply("Pong!");
}

fn owner_check(_context: &Context, message: &Message) -> bool {
    // Replace 7 with your ID
    message.author.id.0 == 7u64
}

fn some_complex_command(context: Context, _message: Message) {
    let _ = context.say("This is a command in a complex group");
}
