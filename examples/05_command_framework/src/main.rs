//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```

#[macro_use]
extern crate serenity;
extern crate typemap;

use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::{Args, DispatchError, StandardFramework, HelpBehaviour, CommandOptions, help_commands};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::Permissions;
use serenity::prelude::Mutex;
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::sync::Arc;
use typemap::Key;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect(
        "Expected a token in the environment",
    );
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    // Commands are equivalent to:
    // "~about"
    // "~emoji cat"
    // "~emoji dog"
    // "~multiply"
    // "~ping"
    // "~some long command"
    client.with_framework(
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        //
        // Refer to the documentation for
        // `serenity::ext::framework::Configuration` for all available
        // configurations.
        StandardFramework::new()
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .prefix("~")
            // You can set multiple delimiters via delimiters()
            // or just one via delimiter(",")
            // If you set multiple delimiters, the order you list them
            // decides their priority (from first to last).
            //
            // In this case, if "," would be first, a message would never
            // be delimited at ", ", forcing you to trim your arguments if you
            // want to avoid whitespaces at the start of each.
            .delimiters(vec![", ", ","]))

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
            let mut data = ctx.data.lock();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.to_string()).or_insert(0);
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
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(|_, _, unknown_command_name| {
            println!("Could not find command named '{}'", unknown_command_name);
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
        .command("about", |c| c.cmd(about))
        // You can use the simple `help(help_commands::with_embeds)` or
        // customise your help-menu via `customised_help()`.
        .customised_help(help_commands::with_embeds, |c| {
                // This replaces the information that a user can pass
                // a command-name as argument to gain specific information about it.
                c.individual_command_tip("Hello! こんにちは！Hola! Bonjour! 您好!\n\
                If you want more information about a specific command, just pass the command as argument.")
                // Some arguments require a `{}` in order to replace it with contextual information.
                // In this case our `{}` refers to a command's name.
                .command_not_found_text("Could not find: `{}`.")
                // On another note, you can set up the help-menu-filter-behaviour.
                // Here are all possible settings shown on all possible options.
                // First case is if a user lacks permissions for a command, we can hide the command.
                .lacking_permissions(HelpBehaviour::Hide)
                // If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
                .lacking_role(HelpBehaviour::Nothing)
                // The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
                .wrong_channel(HelpBehaviour::Strike)
                // Serenity will automatically analyse and generate a hint/tip explaining the possible
                // cases of ~~strikethrough-commands~~, but only if
                // `striked_commands_tip(Some(""))` keeps `Some()` wrapping an empty `String`, which is the default value.
                // If the `String` is not empty, your given `String` will be used instead.
                // If you pass in a `None`, no hint will be displayed at all.
                 })
        .command("commands", |c| c
            // Make this command use the "complicated" bucket.
            .bucket("complicated")
            .cmd(commands))
        .group("Emoji", |g| g
            // Sets a single prefix for a group:
            .prefix("emoji")
            // Sets a command that will be executed if only a group-prefix was passed.
            .default_cmd(dog)
            .command("cat", |c| c
                .desc("Sends an emoji with a cat.")
                .batch_known_as(vec!["kitty", "neko"]) // Adds multiple aliases
                .bucket("emoji") // Make this command use the "emoji" bucket.
                .cmd(cat)
                 // Allow only administrators to call this:
                .required_permissions(Permissions::ADMINISTRATOR))
            .command("dog", |c| c
                .desc("Sends an emoji with a dog.")
                .bucket("emoji")
                .cmd(dog)))
        .group("Math", |g| g
            // Sets multiple prefixes for a group.
            // This requires us to call commands in this group
            // via `~math` (or `~m`) instead of just `~`.
            .prefixes(vec!["m", "math"])
            .command("multiply", |c| c
                .known_as("*") // Lets us also call `~math *` instead of just `~math multiply`.
                .cmd(multiply)))
        .command("latency", |c| c
            .cmd(latency))
        .command("ping", |c| c
            .check(owner_check) // User needs to pass this test to run command
            .cmd(ping))
        .command("role", |c| c
            .cmd(about_role)
            // Limits the usage of this command to roles named:
            .allowed_roles(vec!["mods", "ultimate neko"]))
        .command("some long command", |c| c.cmd(some_long_command))
        .group("Owner", |g| g
            // This check applies to every command on this group.
            // User needs to pass the test for the command to execute.
            .check(admin_check) 
            .command("am i admin", |c| c
                .cmd(am_i_admin))
                .guild_only(true)
        ),
    );

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
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.lock();
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
fn owner_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> bool {
    // Replace 7 with your ID
    msg.author.id == 7
}

// A function which acts as a "check", to determine whether to call a command.
//
// This check analyses whether a guild member permissions has 
// administrator-permissions.
fn admin_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> bool {
    if let Some(member) = msg.member() {

        if let Ok(permissions) = member.permissions() {
            return permissions.administrator();
        }
    }

    false
}

command!(some_long_command(_ctx, msg, args) {
    if let Err(why) = msg.channel_id.say(&format!("Arguments: {}", args.full())) {
        println!("Error sending message: {:?}", why);
    }
});

command!(about_role(_ctx, msg, args) {
    let potential_role_name = args.full();

    if let Some(guild) = msg.guild() {
        // `role_by_name()` allows us to attempt attaining a reference to a role
        // via its name.
        if let Some(role) = guild.read().role_by_name(&potential_role_name) {
            if let Err(why) = msg.channel_id.say(&format!("Role-ID: {}", role.id)) {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    if let Err(why) = msg.channel_id.say(
                      &format!("Could not find role named: {:?}", potential_role_name)) {
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
command!(multiply(_ctx, msg, args) {
    let first = args.single::<f64>().unwrap();
    let second = args.single::<f64>().unwrap();

    let res = first * second;

    if let Err(why) = msg.channel_id.say(&res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});

command!(about(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("This is a small test-bot! : )") {
        println!("Error sending message: {:?}", why);
    }
});

command!(latency(ctx, msg, _args) {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.lock();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply("There was a problem getting the shard manager");

            return Ok(());
        },
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            let _ = msg.reply("No shard found");

            return Ok(());
        },
    };

    let _ = msg.reply(&format!("The shard latency is {:?}", runner.latency));
});

command!(ping(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("Pong! : )") {
        println!("Error sending message: {:?}", why);
    }
});

command!(am_i_admin(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("Yes you are.") {
        println!("Error sending message: {:?}", why);
    }
});

command!(dog(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(":dog:") {
        println!("Error sending message: {:?}", why);
    }
});

command!(cat(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(":cat:") {
        println!("Error sending message: {:?}", why);
    }
});
