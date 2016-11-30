//! Requires the 'methods' feature flag be enabled in your project's Cargo.toml.
//!
//! This can be activated by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/zeyla/serenity.rs.git"
//! features = ["framework", methods"]
//! ```

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
    // "~multiply"
    // "~ping"
    // "~some long command"
    client.with_framework(|f| f
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        //
        // Refer to the documentation for
        // `serenity::ext::framework::Configuration` for all available
        // configurations.
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .prefix("~"))
        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // You can not use this to determine whether a command should be
        // executed. Instead, `set_check` is provided to give you this
        // functionality.
        .before(|_context, message, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     message.author.name);
        })
        // Very similar to `before`, except this will be called directly _after_
        // command execution.
        .after(|_context, _message, command_name| {
            println!("Processed command '{}'", command_name)
        })
        .on("ping", ping_command)
        .set_check("ping", owner_check) // Ensure only the owner can run this
        .on("emoji cat", cat_command)
        .on("emoji dog", dog_command)
        .on("multiply", multiply)
        .on("some long command", some_long_command)
        // Commands can be in closure-form as well.
        //
        // This is not recommended though, as any closure larger than a couple
        // lines will look ugly.
        .on("about", |context, _message, _args| drop(context.say("A test bot"))));

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

// Commands can be created via the `command!` macro, to avoid manually typing
// type annotations.
//
// This may bring more features available for commands in the future. See the
// "multiply" command below for some of the power that the `command!` macro can
// bring.
command!(cat_command(context, _msg, _arg) {
    if let Err(why) = context.say(":cat:") {
        println!("Eror sending message: {:?}", why);
    }
});

fn dog_command(context: &Context, _msg: &Message, _args: Vec<String>) {
    if let Err(why) = context.say(":dog:") {
        println!("Error sending message: {:?}", why);
    }
}

fn ping_command(_context: &Context, message: &Message, _args: Vec<String>) {
    if let Err(why) = message.reply("Pong!") {
        println!("Error sending reply: {:?}", why);
    }
}

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
fn owner_check(_context: &Context, message: &Message) -> bool {
    // Replace 7 with your ID
    message.author.id == 7
}

fn some_long_command(context: &Context, _msg: &Message, args: Vec<String>) {
    if let Err(why) = context.say(&format!("Arguments: {:?}", args)) {
        println!("Error sending message: {:?}", why);
    }
}

// Using the `command!` macro, commands can be created with a certain type of
// "dynamic" type checking. This is a method of requiring that the arguments
// given match the required type, and maps those arguments to the specified
// bindings.
//
// For example, the following will be correctly parsed by the macro:
//
// `~multiply 3.7 4.3`
//
// However, the following will not, as the second argument can not be an f64:
//
// `~multiply 3.7 four`
//
// Since the argument can't be converted, the command returns early.
//
// Additionally, if not enough arguments are given (e.g. `~multiply 3`), then
// the command will return early. If additional arguments are provided, they
// will be ignored.
//
// Argument type overloading is currently not supported.
command!(multiply(context, _message, args, first: f64, second: f64) {
    let res = first * second;

    if let Err(why) = context.say(&res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});
