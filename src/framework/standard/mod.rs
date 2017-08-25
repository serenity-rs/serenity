
pub mod help_commands;

mod command;
mod configuration;
mod create_command;
mod create_group;
mod buckets;
mod args;

pub(crate) use self::buckets::{Bucket, Ratelimit};
pub use self::command::{Command, CommandGroup, CommandType};
pub use self::command::CommandOrAlias;
pub use self::configuration::Configuration;
pub use self::create_command::CreateCommand;
pub use self::create_group::CreateGroup;
pub use self::args::{Args, Error as ArgError};

use self::command::{AfterHook, BeforeHook};
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use client::Context;
use super::Framework;
use model::{ChannelId, GuildId, Message, UserId};
use model::permissions::Permissions;
use tokio_core::reactor::Handle;
use internal::RwLockExt;

#[cfg(feature = "cache")]
use client::CACHE;
#[cfg(feature = "cache")]
use model::Channel;

/// A macro to generate "named parameters". This is useful to avoid manually
/// using the "arguments" parameter and manually parsing types.
///
/// This is meant for use with the command [`Framework`].
///
/// # Examples
///
/// Create a regular `ping` command which takes no arguments:
///
/// ```rust,ignore
/// command!(ping(_context, message, _args) {
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
///     let first = args.single::<i32>().unwrap();
///     let second = args.single::<i32>().unwrap();
///     let product = first * second;
///
///     if let Err(why) = message.reply(&product.to_string()) {
///         println!("Error sending product: {:?}", why);
///     }
/// });
/// ```
///
/// [`Framework`]: framework/index.html
#[macro_export]
macro_rules! command {
    ($fname:ident($c:ident) $b:block) => {
        #[allow(unreachable_code, unused_mut)]
        pub fn $fname(mut $c: &mut $crate::client::Context,
                      _: &$crate::model::Message,
                      _: Args)
                      -> ::std::result::Result<(), String> {
            $b

            Ok(())
        }
    };
    ($fname:ident($c:ident, $m:ident) $b:block) => {
        #[allow(unreachable_code, unused_mut)]
        pub fn $fname(mut $c: &mut $crate::client::Context,
                      $m: &$crate::model::Message,
                      _: Args)
                      -> ::std::result::Result<(), String> {
            $b

            Ok(())
        }
    };
    ($fname:ident($c:ident, $m:ident, $a:ident) $b:block) => {
        #[allow(unreachable_code, unused_mut)]
        pub fn $fname(mut $c: &mut $crate::client::Context,
                      $m: &$crate::model::Message,
                      mut $a: Args)
                      -> ::std::result::Result<(), String> {
            $b

            Ok(())
        }
    };
}

/// A enum representing all possible fail conditions under which a command won't
/// be executed.
pub enum DispatchError {
    /// When a custom function check has failed.
    CheckFailed(Arc<Command>),
    /// When the requested command is disabled in bot configuration.
    CommandDisabled(String),
    /// When the user is blocked in bot configuration.
    BlockedUser,
    /// When the guild or its owner is blocked in bot configuration.
    BlockedGuild,
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

type DispatchErrorHook = Fn(Context, Message, DispatchError) + 'static;

/// A utility for easily managing dispatches to commands.
///
/// Refer to the [module-level documentation] for more information.
///
/// [module-level documentation]: index.html
#[derive(Default)]
pub struct StandardFramework {
    configuration: Configuration,
    groups: HashMap<String, Arc<CommandGroup>>,
    before: Option<Arc<BeforeHook>>,
    dispatch_error_handler: Option<Arc<DispatchErrorHook>>,
    buckets: HashMap<String, Bucket>,
    after: Option<Arc<AfterHook>>,
    /// Whether the framework has been "initialized".
    ///
    /// The framework is initialized once one of the following occurs:
    ///
    /// - configuration has been set;
    /// - a command handler has been set;
    /// - a command check has been set.
    ///
    /// This is used internally to determine whether or not - in addition to
    /// dispatching to the [`EventHandler::on_message`] handler - to have the
    /// framework check if a [`Event::MessageCreate`] should be processed by
    /// itself.
    ///
    /// [`EventHandler::on_message`]:
    /// ../client/event_handler/trait.EventHandler.html#method.on_message
    /// [`Event::MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
    pub initialized: bool,
    user_info: (u64, bool),
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
    /// let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), Handler);
    /// client.with_framework(StandardFramework::new()
    ///     .configure(|c| c
    ///         .depth(3)
    ///         .allow_whitespace(true)
    ///         .prefix("~")));
    /// ```
    ///
    /// [`Client`]: ../client/struct.Client.html
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .bucket("basic", 2, 10, 3)
    ///     .command("ping", |c| c
    ///         .bucket("basic")
    ///         .exec_str("pong!")));
    /// ```
    pub fn bucket<S>(mut self, s: S, delay: i64, time_span: i64, limit: i32) -> Self
        where S: Into<String> {
        self.buckets.insert(
            s.into(),
            Bucket {
                ratelimit: Ratelimit {
                    delay: delay,
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .complex_bucket("basic", 2, 10, 3, |_, guild_id, channel_id, user_id| {
    /// // check if the guild is `123` and the channel where the command(s) was called:
    /// // `456`
    ///         // and if the user who called the command(s) is `789`
    ///         // otherwise don't apply the bucket at all.
    /// guild_id.is_some() && guild_id.unwrap() == 123 && channel_id == 456
    /// && user_id == 789
    ///     })
    ///     .command("ping", |c| c
    ///         .bucket("basic")
    ///         .exec_str("pong!")));
    /// ```
    ///
    /// [`bucket`]: #method.bucket
    #[cfg(feature = "cache")]
    pub fn complex_bucket<S, Check>(mut self,
                                    s: S,
                                    delay: i64,
                                    time_span: i64,
                                    limit: i32,
                                    check: Check)
                                    -> Self
        where Check: Fn(&mut Context, Option<GuildId>, ChannelId, UserId) -> bool + 'static,
              S: Into<String> {
        self.buckets.insert(
            s.into(),
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .complex_bucket("basic", 2, 10, 3, |_, channel_id, user_id| {
    ///         // check if the channel's id where the command(s) was called is `456`
    ///         // and if the user who called the command(s) is `789`
    ///         // otherwise don't apply the bucket at all.
    ///         channel_id == 456 && user_id == 789
    ///     })
    ///     .command("ping", |c| c
    ///         .bucket("basic")
    ///         .exec_str("pong!")));
    /// ```
    ///
    /// [`bucket`]: #method.bucket
    #[cfg(not(feature = "cache"))]
    pub fn complex_bucket<S, Check>(mut self,
                                    s: S,
                                    delay: i64,
                                    time_span: i64,
                                    limit: i32,
                                    check: Check)
                                    -> Self
        where Check: Fn(&mut Context, ChannelId, UserId) -> bool + 'static, S: Into<String> {
        self.buckets.insert(
            s.into(),
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .simple_bucket("simple", 2)
    ///     .command("ping", |c| c
    ///         .bucket("simple")
    ///         .exec_str("pong!")));
    /// ```
    pub fn simple_bucket<S>(mut self, s: S, delay: i64) -> Self
        where S: Into<String> {
        self.buckets.insert(
            s.into(),
            Bucket {
                ratelimit: Ratelimit {
                    delay: delay,
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
        if let Some(Channel::Guild(channel)) = CACHE.read().unwrap().channel(message.channel_id) {
            let guild_id = channel.with(|g| g.guild_id);
            if self.configuration.blocked_guilds.contains(&guild_id) {
                return true;
            }

            if let Some(guild) = guild_id.find() {
                return self.configuration.blocked_users.contains(
                    &guild.with(|g| g.owner_id),
                );
            }
        }

        false
    }

    #[cfg(feature = "cache")]
    fn has_correct_permissions(&self, command: &Arc<Command>, message: &Message) -> bool {
        if !command.required_permissions.is_empty() {
            if let Some(guild) = message.guild() {
                let perms = guild.with(|g| {
                    g.permissions_for(message.channel_id, message.author.id)
                });

                return perms.contains(command.required_permissions);
            }
        }

        true
    }

    #[allow(too_many_arguments)]
    fn should_fail(&mut self,
                   mut context: &mut Context,
                   message: &Message,
                   command: &Arc<Command>,
                   args: &mut Args,
                   to_check: &str,
                   built: &str)
                   -> Option<DispatchError> {
        if self.configuration.ignore_bots && message.author.bot {
            Some(DispatchError::IgnoredBot)
        } else if self.configuration.ignore_webhooks && message.webhook_id.is_some() {
            Some(DispatchError::WebhookAuthor)
        } else if self.configuration.owners.contains(&message.author.id) {
            None
        } else {
            if let Some(ref bucket) = command.bucket {
                if let Some(ref mut bucket) = self.buckets.get_mut(bucket) {
                    let rate_limit = bucket.take(message.author.id.0);
                    match bucket.check {
                        Some(ref check) => {
                            let apply =
                                feature_cache! {{
                                let guild_id = message.guild_id();
                                (check)(context, guild_id, message.channel_id, message.author.id)
                            } else {
                                (check)(context, message.channel_id, message.author.id)
                            }};

                            if apply && rate_limit > 0i64 {
                                return Some(DispatchError::RateLimited(rate_limit));
                            }
                        },
                        None => {
                            if rate_limit > 0i64 {
                                return Some(DispatchError::RateLimited(rate_limit));
                            }
                        },
                    }
                }
            }

            let arg_len = args.len();

            if let Some(x) = command.min_args {
                if arg_len < x as usize {
                    return Some(DispatchError::NotEnoughArguments {
                        min: x,
                        given: arg_len,
                    });
                }
            }

            if let Some(x) = command.max_args {
                if arg_len > x as usize {
                    return Some(DispatchError::TooManyArguments {
                        max: x,
                        given: arg_len,
                    });
                }
            }

            #[cfg(feature = "cache")]
            {
                if self.is_blocked_guild(message) {
                    return Some(DispatchError::BlockedGuild);
                }

                if !self.has_correct_permissions(command, message) {
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
            } else if self.configuration.blocked_users.contains(
                &message.author.id,
            ) {
                Some(DispatchError::BlockedUser)
            } else if self.configuration.disabled_commands.contains(to_check) {
                Some(DispatchError::CommandDisabled(to_check.to_owned()))
            } else if self.configuration.disabled_commands.contains(built) {
                Some(DispatchError::CommandDisabled(built.to_owned()))
            } else {
                let all_passed = command.checks.iter().all(|check| {
                    check(&mut context, message, args, command)
                });

                if all_passed {
                    None
                } else {
                    Some(DispatchError::CheckFailed(command.to_owned()))
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
    /// ```rust
    /// # #[macro_use] extern crate serenity;
    /// #
    /// # fn main() {
    /// # use serenity::prelude::*;
    /// # use serenity::framework::standard::Args;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().on("ping", ping));
    ///
    /// command!(ping(_ctx, msg) {
    ///     let _ = msg.channel_id.say("pong!");
    /// });
    /// # }
    /// ```
    pub fn on<F, S>(mut self, command_name: S, f: F) -> Self
        where F: Fn(&mut Context, &Message, Args) -> Result<(), String> + 'static, S: Into<String> {
        {
            let ungrouped = self.groups.entry("Ungrouped".to_owned()).or_insert_with(
                || {
                    Arc::new(CommandGroup::default())
                },
            );

            if let Some(ref mut group) = Arc::get_mut(ungrouped) {
                let name = command_name.into();

                group.commands.insert(
                    name,
                    CommandOrAlias::Command(
                        Arc::new(Command::new(f)),
                    ),
                );
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
    ///     .description("Responds with 'pong'.")
    ///     .exec(|ctx, _, _| {
    ///         let _ = ctx.say("pong");
    ///     }));
    /// ```
    pub fn command<F, S>(mut self, command_name: S, f: F) -> Self
        where F: FnOnce(CreateCommand) -> CreateCommand, S: Into<String> {
        {
            let ungrouped = self.groups.entry("Ungrouped".to_owned()).or_insert_with(
                || {
                    Arc::new(CommandGroup::default())
                },
            );

            if let Some(ref mut group) = Arc::get_mut(ungrouped) {
                let cmd = f(CreateCommand(Command::default())).0;
                let name = command_name.into();

                if let Some(ref prefix) = group.prefix {
                    for v in &cmd.aliases {
                        group.commands.insert(
                            format!("{} {}", prefix, v),
                            CommandOrAlias::Alias(format!("{} {}", prefix, name)),
                        );
                    }
                } else {
                    for v in &cmd.aliases {
                        group.commands.insert(
                            v.to_owned(),
                            CommandOrAlias::Alias(name.clone()),
                        );
                    }
                }

                group.commands.insert(
                    name,
                    CommandOrAlias::Command(Arc::new(cmd)),
                );
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .group("ping-pong", |g| g
    ///         .command("ping", |c| c.exec_str("pong!"))
    ///         .command("pong", |c| c.exec_str("ping!"))));
    /// ```
    pub fn group<F, S>(mut self, group_name: S, f: F) -> Self
        where F: FnOnce(CreateGroup) -> CreateGroup, S: Into<String> {
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
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
        where F: Fn(Context, Message, DispatchError) + 'static {
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .before(|_, msg, cmd_name| {
    ///         if let Ok(channel) = msg.channel_id.get() {
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
        where F: Fn(&mut Context, &Message, &str) -> bool + 'static {
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
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
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
        where F: Fn(&mut Context, &Message, &str, Result<(), String>) + 'static {
        self.after = Some(Arc::new(f));

        self
    }
}

impl Framework for StandardFramework {
    fn dispatch(&mut self, mut context: Context, message: Message, tokio_handle: &Handle) {
        let res = command::positions(&mut context, &message, &self.configuration);

        let positions = match res {
            Some(mut positions) => {
                // First, take out the prefixes that are as long as _or_ longer
                // than the message, to avoid character boundary violations.
                positions.retain(|p| *p < message.content.len());

                // Ensure that there is _at least one_ position remaining. There
                // is no point in continuing if there is not.
                if positions.is_empty() {
                    return;
                }

                positions
            },
            None => return,
        };

        'outer: for position in positions {
            let mut built = String::new();
            let round = message.content.chars().skip(position).collect::<String>();
            let round = round.trim().split_whitespace().collect::<Vec<&str>>();

            for i in 0..self.configuration.depth {
                if i != 0 {
                    built.push(' ');
                }

                built.push_str(match round.get(i) {
                    Some(piece) => piece,
                    None => continue 'outer,
                });

                let groups = self.groups.clone();

                for group in groups.values() {
                    let command_length = built.len();

                    if let Some(&CommandOrAlias::Alias(ref points_to)) =
                        group.commands.get(&built) {
                        built = points_to.to_owned();
                    }

                    let to_check = if let Some(ref prefix) = group.prefix {
                        if built.starts_with(prefix) && command_length > prefix.len() + 1 {
                            built[(prefix.len() + 1)..].to_owned()
                        } else {
                            continue;
                        }
                    } else {
                        built.clone()
                    };

                    if let Some(&CommandOrAlias::Command(ref command)) =
                        group.commands.get(&to_check) {
                        let before = self.before.clone();
                        let command = command.clone();
                        let after = self.after.clone();
                        let groups = self.groups.clone();

                        let mut args = {
                            let mut content = message.content[position..].trim();
                            content = content[command_length..].trim();

                            let delimiter = self.configuration
                                .delimiters
                                .iter()
                                .find(|&d| content.contains(d))
                                .map_or(" ", |s| s.as_str());

                            Args::new(&content, delimiter)
                        };

                        if let Some(error) = self.should_fail(
                            &mut context,
                            &message,
                            &command,
                            &mut args,
                            &to_check,
                            &built,
                        ) {
                            if let Some(ref handler) = self.dispatch_error_handler {
                                handler(context, message, error);
                            }
                            return;
                        }

                        tokio_handle.spawn_fn(move || {
                            if let Some(before) = before {
                                if !(before)(&mut context, &message, &built) {
                                    return Ok(());
                                }
                            }

                            let result = match command.exec {
                                CommandType::StringResponse(ref x) => {
                                    let _ = message.channel_id.say(x);

                                    Ok(())
                                },
                                CommandType::Basic(ref x) => (x)(&mut context, &message, args),
                                CommandType::WithCommands(ref x) => {
                                    (x)(&mut context, &message, groups, args)
                                },
                            };

                            if let Some(after) = after {
                                (after)(&mut context, &message, &built, result);
                            }

                            Ok(())
                        });

                        return;
                    }
                }
            }
        }
    }

    fn update_current_user(&mut self, user_id: UserId, is_bot: bool) {
        self.user_info = (user_id.0, is_bot);
    }

    fn initialized(&self) -> bool { self.initialized }
}
