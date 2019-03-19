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
use std::{collections::{HashMap, HashSet}, env, fmt::Write, sync::Arc};

use serenity::{
    command,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        Args, CheckResult, CommandOptions, DispatchError, HelpBehaviour,
        help_commands, StandardFramework,
    },
    model::{channel::{Channel, Message}, gateway::Ready, Permissions},
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
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
        let mut data = client.data.write();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    // We will fetch your bot's owners
    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            owners
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };


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
            // A command that will be executed
            // if nothing but a prefix is passed.
            .prefix_only_cmd(about)
            // You can set multiple delimiters via delimiters()
            // or just one via delimiter(",")
            // If you set multiple delimiters, the order you list them
            // decides their priority (from first to last).
            //
            // In this case, if "," would be first, a message would never
            // be delimited at ", ", forcing you to trim your arguments if you
            // want to avoid whitespaces at the start of each.
            .delimiters(vec![", ", ","])
            // Sets the bot's owners. These will be used for commands that
            // are owners only.
            .owners(owners))

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
            let mut data = ctx.data.write();
            let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in ShareMap.");
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
        // Set a function that's called whenever a message is not a command.
        .message_without_command(|_, message| {
            println!("Message is not a command '{}'", message.content);
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|ctx, msg, error| {
            if let DispatchError::RateLimited(seconds) = error {
                let _ = msg.channel_id.say(&ctx.http, &format!("Try this again in {} seconds.", seconds));
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
                // Define the maximum Levenshtein-distance between a searched command-name
                // and commands. If the distance is lower than or equal the set distance,
                // it will be displayed as a suggestion.
                // Setting the distance to 0 will disable suggestions.
                .max_levenshtein_distance(3)
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
                // `strikethrough_commands_tip(Some(""))` keeps `Some()` wrapping an empty `String`, which is the default value.
                // If the `String` is not empty, your given `String` will be used instead.
                // If you pass in a `None`, no hint will be displayed at all.
                 })
        .command("commands", |c| c
            // Make this command use the "complicated" bucket.
            .bucket("complicated")
            .cmd(commands))
        // Command that will repeat passed arguments and remove user and
        // role mentions with safe alternative.
        .command("say", |c| c
            .cmd(say))
        .group("Emoji", |g| g
            // Sets multiple prefixes for a group.
            // This requires us to call commands in this group
            // via `~emoji` (or `~e`) instead of just `~`.
            .prefixes(vec!["emoji", "em"])
            // Set a description to appear if a user wants to display a single group
            // e.g. via help using the group-name or one of its prefixes.
            .desc("A group with commands providing an emoji as response.")
            // Sets a command that will be executed if only a group-prefix was passed.
            .default_cmd(bird)
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
            // Sets a single prefix for this group.
            // So one has to call commands in this group
            // via `~math` instead of just `~`.
            .prefix("math")
            .command("multiply", |c| c
                .known_as("*") // Lets us also call `~math *` instead of just `~math multiply`.
                .cmd(multiply)))
        .command("latency", |c| c
            .cmd(latency))
        .command("ping", |c| c
            // User needs to pass the test for the command to execute.
            .check_customised(admin_check, |c| c
                .name("Admin")
                // Whether the check shall be tested in the help-system.
                .check_in_help(true)
                // Whether the check shall be displayed in the help-system.
                .display_in_help(true))
            .guild_only(true)
            .cmd(ping))
        .command("role", |c| c
            .cmd(about_role)
            // Limits the usage of this command to roles named:
            .allowed_roles(vec!["mods", "ultimate neko"]))
        .command("some long command", |c| c.cmd(some_long_command))
        .group("Owner", |g| g
            .owners_only(true)
            .command("am i admin", |c| c
                .cmd(am_i_admin)
                .guild_only(true))
            .command("slow mode", |c| c
                .cmd(slow_mode)
                .guild_only(true))
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

    let data = ctx.data.read();
    let counter = data.get::<CommandCounter>().expect("Expected CommandCounter in ShareMap.");

    for (k, v) in counter {
        let _ = write!(contents, "- {name}: {amount}\n", name=k, amount=v);
    }

    if let Err(why) = msg.channel_id.say(&ctx.http, &contents) {
        println!("Error sending message: {:?}", why);
    }
});

// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
command!(say(ctx, msg, args) {
    let mut settings = if let Some(guild_id) = msg.guild_id {
       // By default roles, users, and channel mentions are cleaned.
       ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let mut content = content_safe(&ctx.cache, &args.rest(), &settings);

    if let Err(why) = msg.channel_id.say(&ctx.http, &content) {
        println!("Error sending message: {:?}", why);
    }
});

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
fn owner_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    // Replace 7 with your ID to make this check pass.
    //
    // `true` will convert into `CheckResult::Success`,
    //
    // `false` will convert into `CheckResult::Failure(Reason::Unknown)`,
    //
    // and if you want to pass a reason alongside failure you can do:
    // `CheckResult::new_user("Lacked admin permission.")`,
    //
    // if you want to mark it as something you want to log only:
    // `CheckResult::new_log("User lacked admin permission.")`,
    //
    // and if the check's failure origin is unknown you can mark it as such (same as using `false.into`):
    // `CheckResult::new_unknown()`
    (msg.author.id == 7).into()
}

// A function which acts as a "check", to determine whether to call a command.
//
// This check analyses whether a guild member permissions has
// administrator-permissions.
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {

        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }

    false.into()
}

command!(some_long_command(ctx, msg, args) {
    if let Err(why) = msg.channel_id.say(&ctx.http, &format!("Arguments: {:?}", args)) {
        println!("Error sending message: {:?}", why);
    }
});

command!(about_role(ctx, msg, args) {
    let potential_role_name = args.rest();

    if let Some(guild) = msg.guild(&ctx.cache) {
        // `role_by_name()` allows us to attempt attaining a reference to a role
        // via its name.
        if let Some(role) = guild.read().role_by_name(&potential_role_name) {
            if let Err(why) = msg.channel_id.say(&ctx.http, &format!("Role-ID: {}", role.id)) {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Could not find role named: {:?}", potential_role_name)) {
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
command!(multiply(ctx, msg, args) {
    let first = args.single::<f64>()?;
    let second = args.single::<f64>()?;

    let res = first * second;

    if let Err(why) = msg.channel_id.say(&ctx.http, &res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});

command!(about(ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(&ctx.http, "This is a small test-bot! : )") {
        println!("Error sending message: {:?}", why);
    }
});

command!(latency(ctx, msg, _args) {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

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
            let _ = msg.reply(&ctx,  "No shard found");

            return Ok(());
        },
    };

    let _ = msg.reply(&ctx, &format!("The shard latency is {:?}", runner.latency));
});

command!(ping(ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! : )") {
        println!("Error sending message: {:?}", why);
    }
});

command!(am_i_admin(ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Yes you are.") {
        println!("Error sending message: {:?}", why);
    }
});

command!(dog(ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(&ctx.http, ":dog:") {
        println!("Error sending message: {:?}", why);
    }
});

command!(cat(ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(&ctx.http, ":cat:") {
        println!("Error sending message: {:?}", why);
    }
});

command!(bird(ctx, msg, args) {
    let say_content = if args.is_empty() {
        ":bird: can find animals for you.".to_string()
    } else {
        format!(":bird: could not find animal named: `{}`.", args.rest())
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, say_content) {
        println!("Error sending message: {:?}", why);
    }
});

command!(slow_mode(ctx, msg, args) {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {

        if let Err(why) = msg.channel_id.edit(&ctx.http, |c| c.slow_mode_rate(slow_mode_rate_seconds)) {
            println!("Error setting channel's slow mode rate: {:?}", why);

            format!("Failed to set slow mode to `{}` seconds.", slow_mode_rate_seconds)
        } else {
            format!("Successfully set slow mode rate to `{}` seconds.", slow_mode_rate_seconds)
        }
    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache) {
        format!("Current slow mode rate is `{}` seconds.", channel.read().slow_mode_rate.unwrap_or(0))
    } else {
        "Failed to find channel in cache.".to_string()
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, say_content) {
        println!("Error sending message: {:?}", why);
    }
});
