
pub mod help_commands;

mod command;
mod configuration;
mod create_command;
mod create_help_command;
mod create_group;
mod buckets;
mod args;

pub use self::args::{
    Args,
    Iter,
    Error as ArgError
};
pub(crate) use self::buckets::{Bucket, Ratelimit};
pub(crate) use self::command::Help;
pub use self::command::{
    Check,
    HelpFunction,
    HelpOptions,
    Command,
    CommandGroup,
    CommandOptions,
    Error as CommandError
};
pub use self::command::CommandOrAlias;
pub use self::configuration::Configuration;
pub use self::create_help_command::CreateHelpCommand;
pub use self::create_command::{CreateCommand, FnOrCommand};
pub use self::create_group::CreateGroup;

use client::Context;
use internal::RwLockExt;
use model::{
    channel::Message,
    guild::{Guild, Member},
    id::{ChannelId, GuildId, UserId},
    Permissions
};
use self::command::{AfterHook, BeforeHook, MessageWithoutCommandHook, UnrecognisedCommandHook};
use std::{
    collections::HashMap,
    default::Default,
    sync::Arc
};
use super::Framework;
use threadpool::ThreadPool;

#[cfg(feature = "cache")]
use client::CACHE;
#[cfg(feature = "cache")]
use model::channel::Channel;

/// A convenience macro for generating a struct fulfilling the [`Command`][command trait] trait.
///
/// This is meant for use with the [`StandardFramework`], specifically `Framework`::{[`cmd`]/[`command`]}.
///
///
/// If you're just looking for a simple "register this function as a command", use [`StandardFramework::on`].
///
/// # Examples
///
/// Create a regular `ping` command which takes no arguments:
///
/// ```rust,ignore
/// command!(ping(_context, message) {
///     if let Err(why) = message.reply("Pong!") {
///         println!("Error sending pong: {:?}", why);
///     }
/// });
/// ```
///
/// Create a command named `multiply` which accepts 2 floats and multiplies
/// them, sending the product as a reply:
///
/// ```rust,ignore
/// command!(multiply(_context, message, args) {
///     let first = args.single::<f64>().unwrap();
///     let second = args.single::<f64>().unwrap();
///     let product = first * second;
///
///     if let Err(why) = message.reply(&product.to_string()) {
///         println!("Error sending product: {:?}", why);
///     }
/// });
/// ```
///
/// [command trait]: framework/standard/trait.Command.html
/// [`StandardFramework`]: framework/standard/struct.StandardFramework.html
/// [`cmd`]: framework/standard/struct.StandardFramework.html#method.cmd
/// [`command`]: framework/standard/struct.StandardFramework.html#method.command
/// [`StandardFramework::on`]: framework/standard/struct.StandardFramework.html#method.on
#[macro_export]
macro_rules! command {
    ($fname:ident($c:ident) $b:block) => {
        #[allow(non_camel_case_types)]
        pub struct $fname;

        impl $crate::framework::standard::Command for $fname {
            #[allow(unreachable_code, unused_mut)]
            fn execute(&self, mut $c: &mut $crate::client::Context,
                      _: &$crate::model::channel::Message,
                      _: $crate::framework::standard::Args)
                      -> ::std::result::Result<(), $crate::framework::standard::CommandError> {

                $b

                Ok(())
            }
        }
    };
    ($fname:ident($c:ident, $m:ident) $b:block) => {
        #[allow(non_camel_case_types)]
        pub struct $fname;

        impl $crate::framework::standard::Command for $fname {
            #[allow(unreachable_code, unused_mut)]
            fn execute(&self, mut $c: &mut $crate::client::Context,
                      $m: &$crate::model::channel::Message,
                      _: $crate::framework::standard::Args)
                      -> ::std::result::Result<(), $crate::framework::standard::CommandError> {

                $b

                Ok(())
            }
        }
    };
    ($fname:ident($c:ident, $m:ident, $a:ident) $b:block) => {
        #[allow(non_camel_case_types)]
        pub struct $fname;

        impl $crate::framework::standard::Command for $fname {
            #[allow(unreachable_code, unused_mut)]
            fn execute(&self, mut $c: &mut $crate::client::Context,
                      $m: &$crate::model::channel::Message,
                      mut $a: $crate::framework::standard::Args)
                      -> ::std::result::Result<(), $crate::framework::standard::CommandError> {

                $b

                Ok(())
            }
        }
    };
}

macro_rules! command_and_help_args {
    ($message_content:expr, $position:expr, $command_length:expr, $delimiters:expr) => {
        {
            let content = $message_content.chars().skip($position).skip_while(|x| x.is_whitespace())
                .skip($command_length).collect::<String>();

            Args::new(&content.trim(), $delimiters)
        }
    };
}

/// An enum representing all possible fail conditions under which a command won't
/// be executed.
#[derive(Debug)]
pub enum DispatchError {
    /// When a custom function check has failed.
    //
    // TODO: Bring back `Arc<Command>` as `CommandOptions` in 0.6.x.
    CheckFailed,
    /// When the requested command is disabled in bot configuration.
    CommandDisabled(String),
    /// When the user is blocked in bot configuration.
    BlockedUser,
    /// When the guild or its owner is blocked in bot configuration.
    BlockedGuild,
    /// When the channel blocked in bot configuration.
    BlockedChannel,
    /// When the command requester lacks specific required permissions.
    LackOfPermissions(Permissions),
    /// When the command requester has exceeded a ratelimit bucket. The attached
    /// value is the time a requester has to wait to run the command again.
    RateLimited(i64),
    /// When the requested command can only be used in a direct message or group
    /// channel.
    OnlyForDM,
    /// When the requested command can only be ran in guilds, or the bot doesn't
    /// support DMs.
    OnlyForGuilds,
    /// When the requested command can only be used by bot owners.
    OnlyForOwners,
    /// When the requested command requires one role.
    LackingRole,
    /// When there are too few arguments.
    NotEnoughArguments { min: i32, given: usize },
    /// When there are too many arguments.
    TooManyArguments { max: i32, given: usize },
    /// When the command was requested by a bot user when they are set to be
    /// ignored.
    IgnoredBot,
    /// When the bot ignores webhooks and a command was issued by one.
    WebhookAuthor,
}

type DispatchErrorHook = Fn(Context, Message, DispatchError) + Send + Sync + 'static;

/// A utility for easily managing dispatches to commands.
///
/// Refer to the [module-level documentation] for more information.
///
/// [module-level documentation]: index.html
#[derive(Default)]
pub struct StandardFramework {
    configuration: Configuration,
    groups: HashMap<String, Arc<CommandGroup>>,
    help: Option<Arc<Help>>,
    before: Option<Arc<BeforeHook>>,
    dispatch_error_handler: Option<Arc<DispatchErrorHook>>,
    buckets: HashMap<String, Bucket>,
    after: Option<Arc<AfterHook>>,
    unrecognised_command: Option<Arc<UnrecognisedCommandHook>>,
    message_without_command: Option<Arc<MessageWithoutCommandHook>>,
    /// Whether the framework has been "initialized".
    ///
    /// The framework is initialized once one of the following occurs:
    ///
    /// - configuration has been set;
    /// - a command handler has been set;
    /// - a command check has been set.
    ///
    /// This is used internally to determine whether or not - in addition to
    /// dispatching to the [`EventHandler::message`] handler - to have the
    /// framework check if a [`Event::MessageCreate`] should be processed by
    /// itself.
    ///
    /// [`EventHandler::message`]: ../../client/trait.EventHandler.html#method.message
    /// [`Event::MessageCreate`]: ../../model/event/enum.Event.html#variant.MessageCreate
    pub initialized: bool,
    user_id: u64,
}

impl StandardFramework {
    pub fn new() -> Self { StandardFramework::default() }

    /// Configures the framework, setting non-default values. All fields are
    /// optional. Refer to [`Configuration::default`] for more information on
    /// the default values.
    ///
    /// # Examples
    ///
    /// Configuring the framework for a [`Client`], setting the [`depth`] to 3,
    /// [allowing whitespace], and setting the [`prefix`] to `"~"`:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::EventHandler;
    /// # struct Handler;
    /// # impl EventHandler for Handler {}
    /// use serenity::Client;
    /// use serenity::framework::StandardFramework;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN").unwrap();
    /// let mut client = Client::new(&token, Handler).unwrap();
    /// client.with_framework(StandardFramework::new()
    ///     .configure(|c| c
    ///         .depth(3)
    ///         .allow_whitespace(true)
    ///         .prefix("~")));
    /// ```
    ///
    /// [`Client`]: ../../client/struct.Client.html
    /// [`Configuration::default`]: struct.Configuration.html#method.default
    /// [`depth`]: struct.Configuration.html#method.depth
    /// [`prefix`]: struct.Configuration.html#method.prefix
    /// [allowing whitespace]: struct.Configuration.html#method.allow_whitespace
    pub fn configure<F>(mut self, f: F) -> Self
        where F: FnOnce(Configuration) -> Configuration {
        self.configuration = f(self.configuration);

        self
    }

    /// Defines a bucket with `delay` between each command, and the `limit` of uses
    /// per `time_span`.
    ///
    /// # Examples
    ///
    /// Create and use a bucket that limits a command to 3 uses per 10 seconds with
    /// a 2 second delay inbetween invocations:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .bucket("basic", 2, 10, 3)
    ///     .command("ping", |c| c
    ///         .bucket("basic")
    ///         .exec(|_, msg, _| {
    ///             msg.channel_id.say("pong!")?;
    ///
    ///             Ok(())
    ///         })));
    /// ```
    pub fn bucket(mut self, s: &str, delay: i64, time_span: i64, limit: i32) -> Self {
        self.buckets.insert(
            s.to_string(),
            Bucket {
                ratelimit: Ratelimit {
                    delay,
                    limit: Some((time_span, limit)),
                },
                users: HashMap::new(),
                check: None,
            },
        );

        self
    }

    /// Same as [`bucket`] but with a check added.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .complex_bucket("basic", 2, 10, 3, |_, guild_id, channel_id, user_id| {
    ///         // Our bucket is very strict. It cannot apply in DMs.
    ///         // And can only apply if it's in the specific guild, channel and by the specific user.
    ///         guild_id.is_some() && guild_id.unwrap() == 123 && channel_id == 456
    ///         && user_id == 789
    ///     })
    ///     .command("ping", |c| c
    ///         .bucket("basic")
    ///         .exec(|_, msg, _| {
    ///             msg.channel_id.say("pong!")?;
    ///
    ///             Ok(())
    ///         })
    ///     )
    /// );
    /// ```
    ///
    /// [`bucket`]: #method.bucket
    #[cfg(feature = "cache")]
    pub fn complex_bucket<Check>(mut self,
                                    s: &str,
                                    delay: i64,
                                    time_span: i64,
                                    limit: i32,
                                    check: Check)
                                    -> Self
        where Check: Fn(&mut Context, Option<GuildId>, ChannelId, UserId) -> bool
                         + Send
                         + Sync
                         + 'static {
        self.buckets.insert(
            s.to_string(),
            Bucket {
                ratelimit: Ratelimit {
                    delay,
                    limit: Some((time_span, limit)),
                },
                users: HashMap::new(),
                check: Some(Box::new(check)),
            },
        );

        self
    }

    /// Same as [`bucket`] but with a check added.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .complex_bucket("basic", 2, 10, 3, |_, channel_id, user_id| {
    ///         // Our bucket is somewhat strict.
    ///         // It can only apply in the specific channel and by the specific user.
    ///         channel_id == 456 && user_id == 789
    ///     })
    ///     .command("ping", |c| c
    ///         .bucket("basic")
    ///         .exec(|_, msg, _| {
    ///             msg.channel_id.say("pong!")?;
    ///
    ///             Ok(())
    ///         })
    ///     )
    /// );
    /// ```
    ///
    /// [`bucket`]: #method.bucket
    #[cfg(not(feature = "cache"))]
    pub fn complex_bucket<Check>(mut self,
                                    s: &str,
                                    delay: i64,
                                    time_span: i64,
                                    limit: i32,
                                    check: Check)
                                    -> Self
        where Check: Fn(&mut Context, ChannelId, UserId) -> bool + Send + Sync + 'static {
        self.buckets.insert(
            s.to_string(),
            Bucket {
                ratelimit: Ratelimit {
                    delay,
                    limit: Some((time_span, limit)),
                },
                users: HashMap::new(),
                check: Some(Box::new(check)),
            },
        );

        self
    }

    /// Defines a bucket with only a `delay` between each command.
    ///
    /// # Examples
    ///
    /// Create and use a simple bucket that has a 2 second delay between invocations:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .simple_bucket("simple", 2)
    ///     .command("ping", |c| c
    ///         .bucket("simple")
    ///         .exec(|_, msg, _| { msg.channel_id.say("pong!")?; Ok(()) })));
    /// ```
    pub fn simple_bucket(mut self, s: &str, delay: i64) -> Self {
        self.buckets.insert(
            s.to_string(),
            Bucket {
                ratelimit: Ratelimit {
                    delay,
                    limit: None,
                },
                users: HashMap::new(),
                check: None,
            },
        );

        self
    }

    #[cfg(feature = "cache")]
    fn is_blocked_guild(&self, message: &Message) -> bool {
        if let Some(Channel::Guild(channel)) = CACHE.read().channel(message.channel_id) {
            let guild_id = channel.with(|g| g.guild_id);
            if self.configuration.blocked_guilds.contains(&guild_id) {
                return true;
            }

            if let Some(guild) = guild_id.to_guild_cached() {
                return self.configuration
                    .blocked_users
                    .contains(&guild.with(|g| g.owner_id));
            }
        }

        false
    }

    #[cfg(feature = "cache")]
    fn is_blocked_channel(&self, message: &Message) -> bool {
        !self.configuration.allowed_channels.is_empty()
            && !self.configuration
                .allowed_channels
                .contains(&message.channel_id)
    }

    #[inline]
    fn should_fail_common(&self, message: &Message) -> Option<DispatchError> {
        if self.configuration.ignore_bots && message.author.bot {
            Some(DispatchError::IgnoredBot)
        } else if self.configuration.ignore_webhooks && message.webhook_id.is_some() {
            Some(DispatchError::WebhookAuthor)
        } else {
            None
        }
    }

    #[allow(too_many_arguments)]
    #[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
    fn should_fail(&mut self,
                   mut context: &mut Context,
                   message: &Message,
                   command: &Arc<CommandOptions>,
                   group: &Arc<CommandGroup>,
                   args: &mut Args,
                   to_check: &str,
                   built: &str)
                   -> Option<DispatchError> {
        if let Some(err) = self.should_fail_common(message) {
            return Some(err);
        } else {
            let len = args.len();

            if let Some(x) = command.min_args {
                if len < x as usize {
                    return Some(DispatchError::NotEnoughArguments {
                        min: x,
                        given: len,
                    });
                }
            }

            if let Some(x) = command.max_args {
                if len > x as usize {
                    return Some(DispatchError::TooManyArguments {
                        max: x,
                        given: len,
                    });
                }
            }

            if command.owner_privileges && self.configuration.owners.contains(&message.author.id) {
                return None;
            }

            if let Some(ref bucket) = command.bucket {
                if let Some(ref mut bucket) = self.buckets.get_mut(bucket) {
                    let rate_limit = bucket.take(message.author.id.0);

                    // Is there a custom check for when this bucket applies?
                    // If not, assert that it does always.
                    let apply = bucket.check.as_ref().map_or(true, |check| {
                        feature_cache! {{
                            let guild_id = message.guild_id;
                            (check)(context, guild_id, message.channel_id, message.author.id)
                        } else {
                            (check)(context, message.channel_id, message.author.id)
                        }}
                    });

                    if apply && rate_limit > 0i64 {
                        return Some(DispatchError::RateLimited(rate_limit));
                    }
                }
            }

            #[cfg(feature = "cache")]
            {
                if self.is_blocked_guild(message) {
                    return Some(DispatchError::BlockedGuild);
                }

                if self.is_blocked_channel(message) {
                    return Some(DispatchError::BlockedChannel);
                }

                if !has_correct_permissions(command, message) {
                    return Some(DispatchError::LackOfPermissions(
                        command.required_permissions,
                    ));
                }

                if (!self.configuration.allow_dm && message.is_private()) ||
                   (command.guild_only && message.is_private()) {
                    return Some(DispatchError::OnlyForGuilds);
                }

                if command.dm_only && !message.is_private() {
                    return Some(DispatchError::OnlyForDM);
                }
            }

            if command.owners_only {
                Some(DispatchError::OnlyForOwners)
            } else if self.configuration
                   .blocked_users
                   .contains(&message.author.id) {
                Some(DispatchError::BlockedUser)
            } else if self.configuration.disabled_commands.contains(to_check) {
                Some(DispatchError::CommandDisabled(to_check.to_string()))
            } else if self.configuration.disabled_commands.contains(built) {
                Some(DispatchError::CommandDisabled(built.to_string()))
            } else {

                #[cfg(feature = "cache")] {
                    if !command.allowed_roles.is_empty() {
                        if let Some(guild) = message.guild() {
                            let guild = guild.read();

                            if let Some(member) = guild.members.get(&message.author.id) {
                                if let Ok(permissions) = member.permissions() {

                                    if !permissions.administrator()
                                        && !has_correct_roles(command, &guild, member) {
                                        return Some(DispatchError::LackingRole);
                                    }
                                }
                            }
                        }
                    }
                }

                let all_group_checks_passed = group
                    .checks
                    .iter()
                    .all(|check| (check.0)(&mut context, message, args, command));

                if !all_group_checks_passed {
                    return Some(DispatchError::CheckFailed);
                }

                let all_command_checks_passed = command
                    .checks
                    .iter()
                    .all(|check| (check.0)(&mut context, message, args, command));

                if all_command_checks_passed {
                    None
                } else {
                    Some(DispatchError::CheckFailed)
                }
            }
        }
    }

    /// Adds a function to be associated with a command, which will be called
    /// when a command is used in a message.
    ///
    /// This requires that a check - if one exists - passes, prior to being
    /// called.
    ///
    /// Prior to v0.2.0, you will need to use the command builder
    /// via the [`command`] method to set checks. This command will otherwise
    /// only be for simple commands.
    ///
    /// Refer to the [module-level documentation] for more information and
    /// usage.
    ///
    /// [`command`]: #method.command
    /// [module-level documentation]: index.html
    ///
    /// # Examples
    ///
    /// Create and use a simple command:
    ///
    /// ```rust,no_run
    /// # #[macro_use] extern crate serenity;
    /// #
    /// # fn main() {
    /// # use serenity::prelude::*;
    /// # use serenity::framework::standard::Args;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().on("ping", |_, msg, _| {
    ///     msg.channel_id.say("pong!")?;
    ///
    ///     Ok(())
    /// }));
    /// # }
    /// ```
    pub fn on(self, name: &str,
            f: fn(&mut Context, &Message, Args)
            -> Result<(), CommandError>) -> Self {
        self.cmd(name, f)
    }

    /// Same as [`on`], but accepts a [`Command`] directly.
    ///
    /// [`on`]: #method.on
    /// [`Command`]: trait.Command.html
    pub fn cmd<C: Command + 'static>(mut self, name: &str, c: C) -> Self {
        {
            let ungrouped = self.groups
                .entry("Ungrouped".to_string())
                .or_insert_with(|| Arc::new(CommandGroup::default()));

            if let Some(ref mut group) = Arc::get_mut(ungrouped) {
                let cmd: Arc<Command> = Arc::new(c);

                for alias in &cmd.options().aliases {
                     group.commands.insert(
                         alias.to_string(),
                         CommandOrAlias::Alias(name.to_string()),
                     );
                }

                group
                    .commands
                    .insert(name.to_string(), CommandOrAlias::Command(Arc::clone(&cmd)));

                cmd.init();
            }
        }

        self.initialized = true;

        self
    }

    /// Adds a command using command builder.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// framework.command("ping", |c| c
    ///     .desc("Responds with 'pong'.")
    ///     .exec(|ctx, _, _| {
    ///         let _ = ctx.say("pong");
    ///     }));
    /// ```
    pub fn command<F>(mut self, command_name: &str, f: F) -> Self
        where F: FnOnce(CreateCommand) -> CreateCommand {
        {
            let ungrouped = self.groups
                .entry("Ungrouped".to_string())
                .or_insert_with(|| Arc::new(CommandGroup::default()));

            if let Some(ref mut group) = Arc::get_mut(ungrouped) {
                let cmd = f(CreateCommand::default()).finish();
                let name = command_name.to_string();

                if let Some(ref prefixes) = group.prefixes {

                    for v in &cmd.options().aliases {

                        for prefix in prefixes {
                            group.commands.insert(
                                format!("{} {}", prefix, v),
                                CommandOrAlias::Alias(format!("{} {}", prefix, name)),
                            );
                        }
                    }
                } else {
                    for v in &cmd.options().aliases {
                        group
                            .commands
                            .insert(v.to_string(), CommandOrAlias::Alias(name.clone()));
                    }
                }

                group
                    .commands
                    .insert(name, CommandOrAlias::Command(Arc::clone(&cmd)));

                cmd.init();
            }
        }

        self.initialized = true;

        self
    }

    /// Adds a group which can organize several related commands.
    /// Groups are taken into account when using
    /// `serenity::framework::standard::help_commands`.
    ///
    /// # Examples
    ///
    /// Creating a simple group:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .group("ping-pong", |g| g
    ///         .on("ping", |_, msg, _| { msg.channel_id.say("pong!")?; Ok(()) })
    ///         .on("pong", |_, msg, _| { msg.channel_id.say("ping!")?; Ok(()) })));
    /// ```
    pub fn group<F>(mut self, group_name: &str, f: F) -> Self
        where F: FnOnce(CreateGroup) -> CreateGroup {
        let group = f(CreateGroup(CommandGroup::default())).0;

        self.groups.insert(group_name.into(), Arc::new(group));
        self.initialized = true;

        self
    }

    /// Specify the function that's called in case a command wasn't executed for one reason or
    /// another.
    ///
    /// DispatchError represents all possible fail conditions.
    ///
    /// # Examples
    ///
    /// Making a simple argument error responder:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::framework::standard::DispatchError::{NotEnoughArguments,
    /// TooManyArguments};
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .on_dispatch_error(|_, msg, error| {
    ///         match error {
    ///             NotEnoughArguments { min, given } => {
    ///                 let s = format!("Need {} arguments, but only got {}.", min, given);
    ///
    ///                 let _ = msg.channel_id.say(&s);
    ///             },
    ///             TooManyArguments { max, given } => {
    ///                 let s = format!("Max arguments allowed is {}, but got {}.", max, given);
    ///
    ///                 let _ = msg.channel_id.say(&s);
    ///             },
    ///             _ => println!("Unhandled dispatch error."),
    ///         }
    ///     }));
    /// ```
    pub fn on_dispatch_error<F>(mut self, f: F) -> Self
        where F: Fn(Context, Message, DispatchError) + Send + Sync + 'static {
        self.dispatch_error_handler = Some(Arc::new(f));

        self
    }

    /// Specify the function to be called prior to every command's execution.
    /// If that function returns true, the command will be executed.
    ///
    /// # Examples
    ///
    /// Using `before` to log command usage:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .before(|ctx, msg, cmd_name| {
    ///         println!("Running command {}", cmd_name);
    ///         true
    ///     }));
    /// ```
    ///
    /// Using before to prevent command usage:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .before(|_, msg, cmd_name| {
    ///         if let Ok(channel) = msg.channel_id.to_channel() {
    ///             //  Don't run unless in nsfw channel
    ///             if !channel.is_nsfw() {
    ///                 return false;
    ///             }
    ///         }
    ///
    ///         println!("Running command {}", cmd_name);
    ///
    ///         true
    ///     }));
    /// ```
    ///
    pub fn before<F>(mut self, f: F) -> Self
        where F: Fn(&mut Context, &Message, &str) -> bool + Send + Sync + 'static {
        self.before = Some(Arc::new(f));

        self
    }

    /// Specify the function to be called after every command's execution.
    /// Fourth argument exists if command returned an error which you can handle.
    ///
    /// # Examples
    ///
    /// Using `after` to log command usage:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .after(|ctx, msg, cmd_name, error| {
    ///         //  Print out an error if it happened
    ///         if let Err(why) = error {
    ///             println!("Error in {}: {:?}", cmd_name, why);
    ///         }
    ///     }));
    /// ```
    pub fn after<F>(mut self, f: F) -> Self
        where F: Fn(&mut Context, &Message, &str, Result<(), CommandError>) + Send + Sync + 'static {
        self.after = Some(Arc::new(f));

        self
    }

    /// Specify the function to be called if no command could be dispatched.
    ///
    /// # Examples
    ///
    /// Using `unrecognised_command`:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .unrecognised_command(|ctx, msg, unrecognised_command_name| { }));
    /// ```
    pub fn unrecognised_command<F>(mut self, f: F) -> Self
        where F: Fn(&mut Context, &Message, &str) + Send + Sync + 'static {
        self.unrecognised_command = Some(Arc::new(f));

        self
    }

    /// Specify the function to be called if a message contains no command.
    ///
    /// # Examples
    ///
    /// Using `message_without_command`:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .message_without_command(|ctx, msg| { }));
    /// ```
    pub fn message_without_command<F>(mut self, f: F) -> Self
        where F: Fn(&mut Context, &Message) + Send + Sync + 'static {
        self.message_without_command = Some(Arc::new(f));

        self
    }

    /// Sets what code should be executed when a user sends `(prefix)help`.
    ///
    /// If a command named `help` was set with [`command`], then this takes precedence first.
    ///
    /// [`command`]: #method.command
    pub fn help(mut self, f: HelpFunction) -> Self {
        let a = CreateHelpCommand(HelpOptions::default(), f).finish();

        self.help = Some(a);

        self
    }

    /// Sets what code should be executed when sends `(prefix)help`.
    /// Additionally takes a closure with a `CreateHelpCommand` in order
    /// to alter help-commands.
    pub fn customised_help<F>(mut self, f: HelpFunction, c: F) -> Self
        where F: FnOnce(CreateHelpCommand) -> CreateHelpCommand {
        let res = c(CreateHelpCommand(HelpOptions::default(), f));

        self.help = Some(res.finish());

        self
    }
}

fn skip_chars_and_trim_to_new_string(str_to_transform_to_chars: &str, chars_to_skip: usize) -> String {
    let mut chars = str_to_transform_to_chars.chars();

    if chars_to_skip > 0 {
        chars.nth(chars_to_skip - 1);
    }

    chars.as_str().trim().to_string()
}

impl Framework for StandardFramework {
    fn dispatch(
        &mut self,
        mut context: Context,
        message: Message,
        threadpool: &ThreadPool,
    ) {
        let res = command::positions(&mut context, &message, &self.configuration);
        let mut unrecognised_command_name = String::from("");

        let positions = match res {
            Some(mut positions) => {
                // First, take out the prefixes that are as long as _or_ longer
                // than the message, to avoid character boundary violations.
                positions.retain(|p| *p < message.content.len());

                // Ensure that there is _at least one_ position remaining. There
                // is no point in continuing if there is not.
                if positions.is_empty() {

                    if let Some(ref prefix_only_cmd) =
                        self.configuration.prefix_only_cmd {
                        let prefix_only_cmd = Arc::clone(prefix_only_cmd);
                        let before = self.before.clone();
                        let after = self.after.clone();

                        threadpool.execute(move || {
                            if let Some(before) = before {
                                if !(before)(&mut context, &message, "") {
                                    return;
                                }
                            }

                            if !prefix_only_cmd.before(&mut context, &message) {
                                return;
                            }

                            let result = prefix_only_cmd.execute(&mut context,
                                &message, Args::new("", &Vec::new()));

                            prefix_only_cmd.after(&mut context, &message,
                                &result);

                            if let Some(after) = after {
                                (after)(&mut context, &message, "", result);
                            }
                        });
                    }

                    return;
                }

                positions
            },
            None => {
                if let &Some(ref message_without_command) = &self.message_without_command {

                    if !(self.configuration.ignore_bots && message.author.bot) {
                        let message_without_command = message_without_command.clone();
                        threadpool.execute(move || {
                            (message_without_command)(&mut context, &message);
                        });
                    }
                }

                return;
            },
        };

        'outer: for position in positions {
            let mut built = String::new();

            let orginal_round = skip_chars_and_trim_to_new_string(&message.content, position);
            let mut round = orginal_round.split_whitespace();

            for i in 0..self.configuration.depth {
                if i != 0 {
                    built.push(' ');
                }

                built.push_str(match round.next() {
                    Some(piece) => piece,
                    None => continue 'outer,
                });

                let groups = self.groups.clone();

                for group in groups.values() {
                    let command_length = built.len();

                    built = if self.configuration.case_insensitive {
                        built.to_lowercase()
                    } else {
                        built
                    };

                    unrecognised_command_name = built.clone();
                    let cmd = group.commands.get(&built);

                    if let Some(&CommandOrAlias::Alias(ref points_to)) = cmd {
                        built = points_to.to_string();
                    }

                    let mut check_contains_group_prefix = false;
                    let mut longest_matching_prefix_len = 0;
                    let to_check = if let Some(ref prefixes) = group.prefixes {
                        // Once `built` starts with a set prefix,
                        // we want to make sure that all following matching prefixes are longer
                        // than the last matching one, this prevents picking a wrong prefix,
                        // e.g. "f" instead of "ferris" due to "f" having a lower index in the `Vec`.
                        longest_matching_prefix_len = prefixes.iter().fold(0, |longest_prefix_len, prefix|
                            if prefix.len() > longest_prefix_len
                            && built.starts_with(prefix)
                            && (orginal_round.len() == prefix.len() || built.get(prefix.len()..prefix.len() + 1) == Some(" ")) {
                                prefix.len()
                            } else {
                                longest_prefix_len
                            }
                        );

                        if longest_matching_prefix_len == built.len() {
                            check_contains_group_prefix = true;
                            String::new()
                        } else if longest_matching_prefix_len > 0 {
                            check_contains_group_prefix = true;
                            built[longest_matching_prefix_len + 1..].to_string()
                        } else {
                            continue;
                        }
                    } else {
                        built.clone()
                    };

                    let before = self.before.clone();
                    let after = self.after.clone();

                    if to_check == "help" {
                        if let Some(error) = self.should_fail_common(&message) {
                            if let Some(ref handler) = self.dispatch_error_handler {
                                handler(context, message, error);
                            }

                            return;
                        }

                        let help = self.help.clone();

                        if let Some(help) = help {
                            let groups = self.groups.clone();
                            let mut args = command_and_help_args!(&message.content, position, command_length, &self.configuration.delimiters);

                            threadpool.execute(move || {

                                if let Some(before) = before {

                                    if !(before)(&mut context, &message, &built) {
                                        return;
                                    }
                                }

                                let result = (help.0)(&mut context, &message, &help.1, groups, &args);

                                if let Some(after) = after {
                                    (after)(&mut context, &message, &built, result);
                                }
                            });
                            return;
                        }
                    }

                    if !to_check.is_empty() {

                        if let Some(&CommandOrAlias::Command(ref command)) =
                            group.commands.get(&to_check) {
                            let command = Arc::clone(command);
                            let mut args = command_and_help_args!(&message.content, position, command_length, &self.configuration.delimiters);

                            if let Some(error) = self.should_fail(
                                &mut context,
                                &message,
                                &command.options(),
                                &group,
                                &mut args,
                                &to_check,
                                &built,
                            ) {
                                if let Some(ref handler) = self.dispatch_error_handler {
                                    handler(context, message, error);
                                }
                                return;
                            }

                            threadpool.execute(move || {
                                if let Some(before) = before {
                                    if !(before)(&mut context, &message, &built) {
                                        return;
                                    }
                                }

                                if !command.before(&mut context, &message) {
                                    return;
                                }

                                let result = command.execute(&mut context, &message, args);

                                command.after(&mut context, &message, &result);

                                if let Some(after) = after {
                                    (after)(&mut context, &message, &built, result);
                                }
                            });

                            return;

                        }
                    }

                    if check_contains_group_prefix {

                        if let &Some(CommandOrAlias::Command(ref command)) = &group.default_command {
                            let command = Arc::clone(command);
                            let mut args = {
                                Args::new(&orginal_round[longest_matching_prefix_len..], &self.configuration.delimiters)
                            };

                            if let Some(error) = self.should_fail(
                                &mut context,
                                &message,
                                &command.options(),
                                &group,
                                &mut args,
                                &to_check,
                                &built,
                            ) {
                                if let Some(ref handler) = self.dispatch_error_handler {
                                    handler(context, message, error);
                                }
                                return;
                            }

                            threadpool.execute(move || {
                                if let Some(before) = before {
                                    if !(before)(&mut context, &message, &args.full()) {
                                        return;
                                    }
                                }

                                if !command.before(&mut context, &message) {
                                    return;
                                }

                                let result = command.execute(&mut context, &message, args);

                                command.after(&mut context, &message, &result);

                                if let Some(after) = after {
                                    (after)(&mut context, &message, &built, result);
                                }
                            });

                            return;
                        }
                    }
                }
            }
        }

        if !(self.configuration.ignore_bots && message.author.bot) {

            if let &Some(ref unrecognised_command) = &self.unrecognised_command {

                // If both functions are set, we need to clone `Context` and
                // `Message`, else we can avoid it.
                if let &Some(ref message_without_command) = &self.message_without_command {
                    let mut context_unrecognised = context.clone();
                    let message_unrecognised = message.clone();

                    let unrecognised_command = unrecognised_command.clone();
                    threadpool.execute(move || {
                        (unrecognised_command)(&mut context_unrecognised, &message_unrecognised,
                        &unrecognised_command_name);
                    });

                    let message_without_command = message_without_command.clone();
                        threadpool.execute(move || {
                            (message_without_command)(&mut context, &message);
                    });
                } else {
                    let unrecognised_command = unrecognised_command.clone();
                    threadpool.execute(move || {
                        (unrecognised_command)(&mut context, &message, &unrecognised_command_name);
                    });
                }
            } else if let &Some(ref message_without_command) = &self.message_without_command {
                let message_without_command = message_without_command.clone();
                    threadpool.execute(move || {
                        (message_without_command)(&mut context, &message);
                });
            }
        }
    }

    fn update_current_user(&mut self, user_id: UserId) {
        self.user_id = user_id.0;
    }
}

#[cfg(feature = "cache")]
pub fn has_correct_permissions(command: &Arc<CommandOptions>, message: &Message) -> bool {
    if command.required_permissions.is_empty() {
        true
    } else {
        if let Some(guild) = message.guild() {
            let perms = guild
                .with(|g| g.permissions_in(message.channel_id, message.author.id));

            perms.contains(command.required_permissions)
        } else {
            false
        }
    }
}

pub fn has_correct_roles(cmd: &Arc<CommandOptions>, guild: &Guild, member: &Member) -> bool {
    if cmd.allowed_roles.is_empty() {
        true
    } else {
        cmd.allowed_roles
            .iter()
            .flat_map(|r| guild.role_by_name(r))
            .any(|g| member.roles.contains(&g.id))
    }
}

/// Describes the behaviour the help-command shall execute once it encounters
/// a command which the user or command fails to meet following criteria :
/// Lacking required permissions to execute the command.
/// Lacking required roles to execute the command.
/// The command can't be used in the current channel (as in `DM only` or `guild only`).
#[derive(PartialEq, Debug)]
pub enum HelpBehaviour {
    /// Strikes a command by applying `~~{command_name}~~`.
    Strike,
    /// Does not list a command in the help-menu.
    Hide,
    /// The command will be displayed, hence nothing will be done.
    Nothing
}

use std::fmt;

impl fmt::Display for HelpBehaviour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       fmt::Debug::fmt(self, f)
    }
}
