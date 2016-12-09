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
extern crate typemap;

use serenity::client::Context;
use serenity::Client;
use serenity::model::Message;
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
    let mut client = Client::login_bot(&token);

    {
        let mut data = client.data.lock().unwrap();
        data.insert::<CommandCounter>(HashMap::default());
    }

    client.on_ready(|_, ready| {
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
        .before(|context, message, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     message.author.name);

            // Increment the number of times this command has been run once. If
            // the command's name does not exist in the counter, add a default
            // value of 0.
            let mut data = context.data.lock().unwrap();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.clone()).or_insert(0);
            *entry += 1;
        })
        // Very similar to `before`, except this will be called directly _after_
        // command execution.
        .after(|_, _, command_name| {
            println!("Processed command '{}'", command_name)
        })
        .command("about", |c| c.exec_str("A test bot"))
        .command("commands", |c| c
            .check(owner_check)
            .exec(commands))
        .command("emoji cat", |c| c.exec_str(":cat:"))
        .command("emoji dog", |c| c.exec_str(":dog:"))
        .command("multiply", |c| c.exec(multiply))
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
command!(commands(context, _msg, _args) {
    let mut contents = "Commands used:\n".to_owned();

    let data = context.data.lock().unwrap();
    let counter = data.get::<CommandCounter>().unwrap();

    for (k, v) in counter {
        let _ = write!(contents, "- {name}: {amount}\n", name=k, amount=v);
    }

    if let Err(why) = context.say(&contents) {
        println!("Error sending message: {:?}", why);
    }
});

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
fn owner_check(_: &Context, message: &Message) -> bool {
    // Replace 7 with your ID
    message.author.id == 7
}

fn some_long_command(context: &Context, _: &Message, args: Vec<String>) {
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
command!(multiply(context, _msg, args, first: f64, second: f64) {
    let res = first * second;

    if let Err(why) = context.say(&res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});
