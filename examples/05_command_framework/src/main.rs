//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/zeyla/serenity.git"
//! features = ["framework"]
//! ```

#[macro_use]
extern crate serenity;
extern crate typemap;

use serenity::client::Context;
use serenity::Client;
use serenity::model::{Message, permissions};
use serenity::ext::framework::{DispatchError, help_commands};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use typemap::Key;

struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token);

    {
        let mut data = client.data.lock().unwrap();
        data.insert::<CommandCounter>(HashMap::default());
    }

    client.on_ready(|_ctx, ready| {
        println!("{} is connected!", ready.user.name);
    });

    // Commands are equivalent to:
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
        .before(|ctx, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);

            // Increment the number of times this command has been run once. If
            // the command's name does not exist in the counter, add a default
            // value of 0.
            let mut data = ctx.data.lock().unwrap();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.clone()).or_insert(0);
            *entry += 1;

            true // if `before` returns false, command processing doesn't happen.
        })
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            }
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|_ctx, msg, error| {
            if let DispatchError::RateLimited(seconds) = error {
                let _ = msg.channel_id.say(&format!("Try this again in {} seconds.", seconds));
            }
        })
        // Can't be used more than once per 5 seconds:
        .simple_bucket("emoji", 5)
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay:
        .bucket("complicated", 5, 30, 2)
        .command("about", |c| c.exec_str("A test bot"))
        .command("help", |c| c.exec_help(help_commands::plain))
        .command("commands", |c| c
            // Make this command use the "complicated" bucket.
            .bucket("complicated")
            .exec(commands))
        .group("Emoji", |g| g
            .prefix("emoji")
            .command("cat", |c| c
                .desc("Sends an emoji with a cat.")
                .batch_known_as(vec!["kitty", "neko"]) // Adds multiple aliases
                .bucket("emoji") // Make this command use the "emoji" bucket.
                .exec_str(":cat:")
                 // Allow only administrators to call this:
                .required_permissions(permissions::ADMINISTRATOR))
            .command("dog", |c| c
                .desc("Sends an emoji with a dog.")
                .bucket("emoji")
                .exec_str(":dog:")))
        .command("multiply", |c| c
            .known_as("*") // Lets us call ~* instead of ~multiply
            .exec(multiply))
        .command("ping", |c| c
            .check(owner_check)
            .exec_str("Pong!"))
        .command("some long command", |c| c.exec(some_long_command)));

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
command!(commands(ctx, msg, _args) {
    let mut contents = "Commands used:\n".to_owned();

    let data = ctx.data.lock().unwrap();
    let counter = data.get::<CommandCounter>().unwrap();

    for (k, v) in counter {
        let _ = write!(contents, "- {name}: {amount}\n", name=k, amount=v);
    }

    if let Err(why) = msg.channel_id.say(&contents) {
        println!("Error sending message: {:?}", why);
    }
});

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
fn owner_check(_: &mut Context, msg: &Message) -> bool {
    // Replace 7 with your ID
    msg.author.id == 7
}

command!(some_long_command(_ctx, msg, args) {
    if let Err(why) = msg.channel_id.say(&format!("Arguments: {:?}", args)) {
        println!("Error sending message: {:?}", why);
    }
});

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
command!(multiply(_ctx, msg, args, first: f64, second: f64) {
    let res = first * second;

    if let Err(why) = msg.channel_id.say(&res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});
