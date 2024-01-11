//! Requires the 'framework' feature flag be enabled in your project's `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
#![allow(deprecated)] // We recommend migrating to poise, instead of using the standard command framework.
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write;
use std::sync::Arc;

use serenity::async_trait;
use serenity::builder::EditChannel;
use serenity::framework::standard::buckets::{LimitedFor, RevertBucket};
use serenity::framework::standard::macros::{check, command, group, help, hook};
use serenity::framework::standard::{
    help_commands,
    Args,
    BucketBuilder,
    CommandGroup,
    CommandOptions,
    CommandResult,
    Configuration,
    DispatchError,
    HelpOptions,
    Reason,
    StandardFramework,
};
use serenity::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};

// A container type is created for inserting into the Client's `data`, which allows for data to be
// accessible across all events and framework commands, or anywhere else that has a copy of the
// `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(about, am_i_admin, say, commands, ping, latency, some_long_command, upper_command)]
struct General;

#[group]
// Sets multiple prefixes for a group.
// This requires us to call commands in this group via `~emoji` (or `~em`) instead of just `~`.
#[prefixes("emoji", "em")]
// Set a description to appear if a user wants to display a single group e.g. via help using the
// group-name or one of its prefixes.
#[description = "A group with commands providing an emoji as response."]
// Summary only appears when listing multiple groups.
#[summary = "Do emoji fun!"]
// Sets a command that will be executed if only a group-prefix was passed.
#[default_command(bird)]
#[commands(cat, dog)]
struct Emoji;

#[group]
// Sets a single prefix for this group.
// So one has to call commands in this group via `~math` instead of just `~`.
#[prefix = "math"]
#[commands(multiply)]
struct Math;

#[group]
#[owners_only]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Summary only appears when listing multiple groups.
#[summary = "Commands for server owners"]
#[commands(slow_mode)]
struct Owner;

// The framework provides two built-in help commands for you to use. But you can also make your own
// customized help command that forwards to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass a command-name as argument to gain specific
// information about it.
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name and commands. If the
// distance is lower than or equal the set distance, it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate how deeply an item
// is indented. The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible cases of
// ~~strikethrough-commands~~, but only if `strikethrough_commands_tip_in_{dm, guild}` aren't
// specified. If you pass in a value, it will be displayed instead.
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If the command's name does not
    // exist in the counter, add a default value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{command_name}'"),
        Err(why) => println!("Command '{command_name}' returned error {why:?}"),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{unknown_command_name}'");
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use serenity::futures::future::BoxFuture;
use serenity::FutureExt;
fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
    _command_name: &str,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, format!("Try this again in {} seconds.", info.as_secs()))
                    .await;
            }
        };
    }
    .boxed()
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        // Set a function to be called prior to each command execution. This provides the context
        // of the command, the message that was received, and the full name of the command that
        // will be called.
        //
        // Avoid using this to determine whether a specific command should be executed. Instead,
        // prefer using the `#[check]` macro which gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your application if you are
        // fine using nightly Rust. If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
        .before(before)
        // Similar to `before`, except will be called directly _after_ command execution.
        .after(after)
        // Set a function that's called whenever an attempted command-call's command could not be
        // found.
        .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command can
        // only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        // Can't be used more than once per 5 seconds:
        .bucket("emoji", BucketBuilder::default().delay(5)).await
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per
        // channel. Optionally `await_ratelimits` will delay until the command can be executed
        // instead of cancelling the command invocation.
        .bucket("complicated",
            BucketBuilder::default().limit(2).time_span(30).delay(5)
                // The target each bucket will apply to.
                .limit_for(LimitedFor::Channel)
                // The maximum amount of command invocations that can be delayed per target.
                // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
                .await_ratelimits(1)
                // A function to call when a rate limit leads to a delay.
                .delay_action(delay_action)
        ).await
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&EMOJI_GROUP)
        .group(&MATH_GROUP)
        .group(&OWNER_GROUP);

    framework.configure(
        Configuration::new().with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("~")
            // In this case, if "," would be first, a message would never be delimited at ", ",
            // forcing you to trim your arguments if you want to avoid whitespaces at the start of
            // each.
            .delimiters(vec![", ".into(), ",".into()])
            // Sets the bot's owners. These will be used for commands that are owners only.
            .owners(owners),
    );

    // For this example to run properly, the "Presence Intent" and "Server Members Intent" options
    // need to be enabled.
    // These are needed so the `required_permissions` macro works on the commands that need to use
    // it.
    // You will need to enable these 2 options on the bot application, and possibly wait up to 5
    // minutes.
    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

// Commands can be created via the attribute `#[command]` macro.
#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");

    for (name, amount) in counter {
        writeln!(contents, "- {name}: {amount}")?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

// Repeats what the user passed as argument but ensures that user and role mentions are replaced
// with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(mut x) => {
            // We only need to wipe mentions in guilds, as DM mentions do not matter.
            if let Some(guild) = msg.guild(&ctx.cache) {
                // By default roles, users, and channel mentions are cleaned.
                let settings = ContentSafeOptions::default()
                    // We do not want to clean channel mentions as they do not ping users.
                    .clean_channel(false);

                x = content_safe(&guild, &x, settings, &msg.mentions);
            }

            msg.channel_id.say(&ctx.http, x).await?;
            return Ok(());
        },
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.").await?;
            return Ok(());
        },
    };
}

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message in order for the
// command to be executed. If the check fails, the command is not called.
#[check]
#[name = "Owner"]
#[rustfmt::skip]
async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    // Replace 7 with your ID to make this check pass.
    //
    // 1. If you want to pass a reason alongside failure you can do:
    //    `Reason::User("Lacked admin permission.".to_string())`,
    //
    // 2. If you want to mark it as something you want to log only:
    //    `Reason::Log("User lacked admin permission.".to_string())`,
    //
    // 3. If the check's failure origin is unknown you can mark it as such:
    //    `Reason::Unknown`
    //
    // 4. If you want log for your system and for the user, use:
    //    `Reason::UserAndLog { user, log }`
    if msg.author.id != 7 {
        return Err(Reason::User("Lacked owner permission".to_string()));
    }

    Ok(())
}

#[command]
async fn some_long_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, format!("Arguments: {:?}", args.rest())).await?;

    Ok(())
}

#[command]
// Limits the usage of this command to roles named:
#[allowed_roles("mods", "ultimate neko")]
async fn about_role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let role_name = args.rest();
    let to_send = match msg.guild(&ctx.cache).as_deref().and_then(|g| g.role_by_name(role_name)) {
        Some(role_id) => format!("Role-ID: {role_id}"),
        None => format!("Could not find role name: {role_name:?}"),
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, to_send).await {
        println!("Error sending message: {why:?}");
    }

    Ok(())
}

#[command]
// Lets us also call `~math *` instead of just `~math multiply`.
#[aliases("*")]
async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let first = args.single::<f64>()?;
    let second = args.single::<f64>()?;

    let res = first * second;

    msg.channel_id.say(&ctx.http, &res.to_string()).await?;

    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "This is a small test-bot! : )").await?;

    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and retrieving
    // information about shards.
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager").await?;

            return Ok(());
        },
    };

    let runners = shard_manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events over the shard, so
    // we'll get the information about the shard runner for the shard this command was sent over.
    let runner = match runners.get(&ctx.shard_id) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        },
    };

    msg.reply(ctx, format!("The shard latency is {:?}", runner.latency)).await?;

    Ok(())
}

#[command]
// Limit command usage to guilds.
#[only_in(guilds)]
#[checks(Owner)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong! : )").await?;

    Ok(())
}

#[command]
// Adds multiple aliases
#[aliases("kitty", "neko")]
// Make this command use the "emoji" bucket.
#[bucket = "emoji"]
// Allow only administrators to call this:
#[required_permissions("ADMINISTRATOR")]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":cat:").await?;

    // We can return one ticket to the bucket undoing the ratelimit.
    Err(RevertBucket.into())
}

#[command]
#[description = "Sends an emoji with a dog."]
#[bucket = "emoji"]
async fn dog(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":dog:").await?;

    Ok(())
}

#[command]
async fn bird(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if args.is_empty() {
        ":bird: can find animals for you.".to_string()
    } else {
        format!(":bird: could not find animal named: `{}`.", args.rest())
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}

// We could also use #[required_permissions(ADMINISTRATOR)] but that would not let us reply when it
// fails.
#[command]
async fn am_i_admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let is_admin = if let (Some(member), Some(guild)) = (&msg.member, msg.guild(&ctx.cache)) {
        member.roles.iter().any(|role| {
            guild.roles.get(role).is_some_and(|r| r.has_permission(Permissions::ADMINISTRATOR))
        })
    } else {
        false
    };

    if is_admin {
        msg.channel_id.say(&ctx.http, "Yes, you are.").await?;
    } else {
        msg.channel_id.say(&ctx.http, "No, you are not.").await?;
    }

    Ok(())
}

#[command]
async fn slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u16>() {
        let builder = EditChannel::new().rate_limit_per_user(slow_mode_rate_seconds);
        if let Err(why) = msg.channel_id.edit(&ctx.http, builder).await {
            println!("Error setting channel's slow mode rate: {why:?}");

            format!("Failed to set slow mode to `{slow_mode_rate_seconds}` seconds.")
        } else {
            format!("Successfully set slow mode rate to `{slow_mode_rate_seconds}` seconds.")
        }
    } else if let Some(channel) = msg.channel_id.to_channel_cached(&ctx.cache) {
        if let Some(slow_mode_rate) = channel.rate_limit_per_user {
            format!("Current slow mode rate is `{slow_mode_rate}` seconds.")
        } else {
            "There is no current slow mode rate for this channel.".to_string()
        }
    } else {
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}

// A command can have sub-commands, just like in command lines tools. Imagine `cargo help` and
// `cargo help run`.
#[command("upper")]
#[sub_commands(sub)]
async fn upper_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main function!").await?;

    Ok(())
}

// This will only be called if preceded by the `upper`-command.
#[command]
#[aliases("sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn sub(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;

    Ok(())
}
