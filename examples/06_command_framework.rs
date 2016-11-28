#[macro_use]
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

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    // Commands are equivilant to:
    // "~about"
    // "~emoji cat"
    // "~emoji dog"
    // "~ping"
    // "~some complex command"
    client.with_framework(|f| f
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .prefix("~"))
        .before(|_context, message, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     message.author.name);
        })
        .after(|_context, _message, command_name| {
            println!("Processed command '{}'", command_name)
        })
        .on("ping", ping_command)
        .set_check("ping", owner_check) // Ensure only the owner can run this
        .on("emoji cat", cat_command)
        .on("emoji dog", dog_command)
        .on("multiply", multiply)
        .on("some complex command", some_complex_command)
        // Commands can be in closure-form as well
        .on("about", |context, _message, _args| drop(context.say("A test bot"))));

    let _ = client.start();
}

command!(cat_command(context, _msg, _arg) {
    let _ = context.say(":cat:");
});

fn dog_command(context: &Context, _msg: &Message, _args: Vec<String>) {
    let _ = context.say(":dog:");
}

fn ping_command(_context: &Context, message: &Message, _args: Vec<String>) {
    let _ = message.reply("Pong!");
}

fn owner_check(_context: &Context, message: &Message) -> bool {
    // Replace 7 with your ID
    message.author.id == 7
}

fn some_complex_command(context: &Context, _msg: &Message, args: Vec<String>) {
    let _ = context.say(&format!("Arguments: {:?}", args));
}

command!(multiply(context, _message, args, first: f64, second: f64) {
    let res = first * second;

    let _ = context.say(&res.to_string());

    println!("{:?}", args);
});
