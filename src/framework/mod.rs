//! The framework is a customizable method of separating commands.
//!
//! This is used in combination with [`Client::with_framework`].
//!
//! The framework has a number of configurations, and can have any number of
//! commands bound to it. The primary purpose of it is to offer the utility of
//! not needing to manually match message content strings to determine if a
//! message is a command.
//!
//! Additionally, "checks" can be added to commands, to ensure that a certain
//! condition is met prior to calling a command; this could be a check that the
//! user who posted a message owns the bot, for example.
//!
//! Each command has a given named, and an associated function/closure. For
//! example, you might have two commands: `"ping"` and `"weather"`. These each
//! have an associated function that are called if the framework determines
//! that a message is of that command.
//!
//! Assuming a command prefix of `"~"`, then the following would occur with the
//! two previous commands:
//!
//! ```ignore
//! ~ping // calls the ping command's function
//! ~pin // does not
//! ~ ping // _does_ call it _if_ the `allow_whitespace` option is enabled
//! ~~ping // does not
//! ```
//!
//! # Examples
//!
//! Configuring a Client with a framework, which has a prefix of `"~"` and a
//! ping and about command:
//!
//! ```rust,ignore
//! extern crate serenity;
//! extern crate serenity_attributes;
//!
//! use serenity_attributes::command;
//! use serenity::client::{Client, Context};
//! use serenity::model::Message;
//! use std::env;
//!
//! let mut client = Client::login(&env::var("DISCORD_BOT_TOKEN").unwrap());
//!
//! client.with_framework(|f| f
//!     .configure(|c| c.prefix("~"))
//!     .register(About)
//!     .register(Ping));
//!
//! #[command]
//! fn about() -> String {
//!    "A simple test bot".to_string()
//! }
//! 
//! #[command]
//! fn ping(msg: &Message) {
//!     let _ = msg.channel_id.say("Pong!");
//! }
//! ```
//!
//! A command with named args:
//! ```rust,ignore
//! #[macro_use]
//! extern crate serenity; 
//! extern crate serenity_attributes;
//!
//! use serenity_attributes::command;
//! use serenity::client::{Client, Context};
//! use serenity::model::Message;
//! use std::env;
//!
//! let mut client = Client::login(&env::var("DISCORD_BOT_TOKEN").unwrap());
//!
//! client.with_framework(|f| f
//!     .configure(|c| c.prefix("~"))
//!     .register(Number));
//!
//! #[command]
//! fn number(msg: &Message, args: Vec<String>) {
//!    named_args!(args, number:i32;);
//!    let _ = msg.channel_id.say(&format!("The number you passed: {}", number));
//! }
//! ```
//!  
//! [`Client::with_framework`]: ../../client/struct.Client.html#method.with_framework

pub mod help_commands;

mod command;
mod configuration;
mod buckets;

pub use self::buckets::{Bucket, MemberRatelimit, Ratelimit};
pub use self::command::{Command, CommandGroup};
pub use self::configuration::Configuration;


use self::command::{AfterHook, BeforeHook};
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use std::thread;
use ::client::Context;
use ::model::{Message, UserId};
use ::model::permissions::Permissions;
use ::utils;

#[cfg(feature="cache")]
use ::client::CACHE;
#[cfg(feature="cache")]
use ::model::Channel;

/// A enum representing all possible fail conditions under which a command won't
/// be executed.
pub enum DispatchError {
    /// When a custom function check has failed.
    CheckFailed,
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

type DispatchErrorHook = Fn(Context, Message, DispatchError) + Send + Sync + 'static;

/// A utility for easily managing dispatches to commands.
///
/// Refer to the [module-level documentation] for more information.
///
/// [module-level documentation]: index.html
#[allow(type_complexity)]
pub struct Framework<T: Command> {
    configuration: Configuration,
    #[doc(hidden)]
    pub groups: HashMap<String, Arc<CommandGroup<T>>>,
    #[doc(hidden)]
    pub aliases: HashMap<String, String>,
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
    /// dispatching to the [`Client::on_message`] handler - to have the
    /// framework check if a [`Event::MessageCreate`] should be processed by
    /// itself.
    ///
    /// [`Client::on_message`]: ../client/struct.Client.html#method.on_message
    /// [`Event::MessageCreate`]: ../model/event/enum.Event.html#variant.MessageCreate
    pub initialized: bool,
    user_info: (u64, bool),
}

// This's to get around the `the trait bound `T: std::default::Default` is not satisfied` error.
impl<T: Command> Default for Framework<T> {
    fn default() -> Self {
        Framework {
            ..Default::default()
        }
    }
}

impl<T: Command + Send + Sync + 'static + Clone> Framework<T> {
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
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN").unwrap());
    /// client.with_framework(|f| f
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

    /// Registers the provided command to the framework.
    /// If the `#[group(name)]` attribute was used when creating a command,
    /// that name'll be used for assigning the command to the group, if not, 
    /// "Ungrouped"'s used instead.
    ///
    /// # Examples
    /// ```rust,no_run
    /// extern crate serenity;
    /// extern crate serenity_attributes;
    ///
    /// use serenity::Client;
    /// use serenity_attributes::command;
    ///
    /// #[command]
    /// #[group(abc, prefix = "abc")]
    /// fn some_command(msg: &Message) {
    ///     let _ = msg.channel_id.say("hello");
    /// }
    ///  
    /// let mut client = Client::login(&env::var("DISCORD_TOKEN").unwrap());
    /// client.with_framework(|f| 
    ///     f.configure(|c| c.prefix("~"))
    ///     .register(SomeCommand)); // `~abc some_command` => `hello`
    pub fn register(mut self, command: T) -> Self {
        for alias in &command.aliases() {
            self.aliases.insert(alias.clone(), command.name());
        }

        if self.groups.contains_key(&command.group()) {
            if let Some(g) = self.groups.get_mut(&command.group()) {
                Arc::make_mut(g).insert(command);
            }
            return self;
        }
        let mut command_group = CommandGroup::new();
        command_group.insert(command.clone());
        if !command.group_prefix().is_empty() {
            command_group.set_prefix(command.group_prefix());
        }
        self.groups.insert(command.group(), Arc::new(command_group));
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
    /// # use serenity::Client;
    /// # let mut client = Client::login("token");
    /// #
    /// client.with_framework(|f| f
    ///     .bucket("basic", 2, 10, 3)
    ///     .command("ping", |c| c
    ///         .bucket("basic")
    ///         .exec_str("pong!")));
    /// ```
    pub fn bucket<S>(mut self, s: S, delay: i64, time_span: i64, limit: i32) -> Self
        where S: Into<String> {
        self.buckets.insert(s.into(), Bucket {
            ratelimit: Ratelimit {
                delay: delay,
                limit: Some((time_span, limit)),
            },
            users: HashMap::new(),
        });

        self
    }

    /// Defines a bucket with only a `delay` between each command.
    ///
    /// # Examples
    ///
    /// Create and use a simple bucket that has a 2 second delay between invocations:
    ///
    /// ```rust
    /// # use serenity::Client;
    /// # let mut client = Client::login("token");
    /// #
    /// client.with_framework(|f| f
    ///     .simple_bucket("simple", 2)
    ///     .command("ping", |c| c
    ///         .bucket("simple")
    ///         .exec_str("pong!")));
    /// ```
    pub fn simple_bucket<S>(mut self, s: S, delay: i64) -> Self
        where S: Into<String> {
        self.buckets.insert(s.into(), Bucket {
            ratelimit: Ratelimit {
                delay: delay,
                limit: None,
            },
            users: HashMap::new(),
        });

        self
    }

    #[cfg(feature="cache")]
    fn is_blocked_guild(&self, message: &Message) -> bool {
        if let Some(Channel::Guild(channel)) = CACHE.read().unwrap().channel(message.channel_id) {
            let guild_id = channel.read().unwrap().guild_id;
            if self.configuration.blocked_guilds.contains(&guild_id) {
                return true;
            }

            if let Some(guild) = guild_id.find() {
                return self.configuration.blocked_users.contains(&guild.read().unwrap().owner_id);
            }
        }

        false
    }

    #[cfg(feature="cache")]
    fn has_correct_permissions(&self, command: &Arc<T>, message: &Message) -> bool {
        if !command.required_permissions().is_empty() {
            if let Some(guild) = message.guild() {
                let perms = guild.read().unwrap().permissions_for(message.channel_id, message.author.id);

                return perms.contains(command.required_permissions());
            }
        }

        true
    }

    fn checks_passed(&self, command: &Arc<T>, mut context: &mut Context, message: &Message) -> bool {
        for check in &command.checks() {
            if !(check)(&mut context, message) {
                return false;
            }
        }

        true
    }

    #[allow(too_many_arguments)]
    fn should_fail(&mut self,
                   mut context: &mut Context,
                   message: &Message,
                   command: &Arc<T>,
                   args: usize,
                   to_check: &str,
                   built: &str) -> Option<DispatchError> {
        if self.configuration.ignore_bots && message.author.bot {
            Some(DispatchError::IgnoredBot)
        } else if self.configuration.ignore_webhooks && message.webhook_id.is_some() {
            Some(DispatchError::WebhookAuthor)
        } else if self.configuration.owners.contains(&message.author.id) {
            None
        } else {
            if let Some(rate_limit) = command.bucket().clone().map(|mut x| x.take(message.author.id.0)) {
                if rate_limit > 0i64 {
                    return Some(DispatchError::RateLimited(rate_limit));
                }
            }

            if let Some(x) = command.min_args() {
                if args < x as usize {
                    return Some(DispatchError::NotEnoughArguments {
                        min: x,
                        given: args
                    });
                }
            }

            if let Some(x) = command.max_args() {
                if args > x as usize {
                    return Some(DispatchError::TooManyArguments {
                        max: x,
                        given: args
                    });
                }
            }

            #[cfg(feature="cache")]
            {
                if self.is_blocked_guild(message) {
                    return Some(DispatchError::BlockedGuild);
                }

                if !self.has_correct_permissions(command, message) {
                    return Some(DispatchError::LackOfPermissions(command.required_permissions()));
                }

                if (!self.configuration.allow_dm && message.is_private()) ||
                    (command.guild_only() && message.is_private()) {
                    return Some(DispatchError::OnlyForGuilds);
                }

                if command.dm_only() && !message.is_private() {
                    return Some(DispatchError::OnlyForDM);
                }
            }

            if command.owners_only() {
                Some(DispatchError::OnlyForOwners)
            } else if !self.checks_passed(command, &mut context, message) {
                Some(DispatchError::CheckFailed)
            } else if self.configuration.blocked_users.contains(&message.author.id) {
                Some(DispatchError::BlockedUser)
            } else if (!self.configuration.allow_dm && message.is_private()) ||
                (message.is_private() && command.guild_only()) {
                Some(DispatchError::OnlyForGuilds)
            } else if self.configuration.disabled_commands.contains(to_check) {
                Some(DispatchError::CommandDisabled(to_check.to_owned()))
            } else if self.configuration.disabled_commands.contains(built) {
                Some(DispatchError::CommandDisabled(built.to_owned()))
            } else if !message.is_private() && command.dm_only() {
                Some(DispatchError::OnlyForDM)
            } else {
                None
            }
        }
    }

    #[allow(cyclomatic_complexity)]
    #[doc(hidden)]
    pub fn dispatch(&mut self, mut context: Context, message: Message) {
        let res = command::positions(&mut context, &message.content, &self.configuration);

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
            let round = message.content.chars()
                .skip(position)
                .collect::<String>();
            let round = round.trim()
                .split_whitespace()
                .collect::<Vec<&str>>();

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

                    if let Some(ref points_to) = self.aliases.get(&built) {
                        built = (*points_to).clone();
                    }

                    let to_check = if !group.prefix.is_empty() {
                        if built.starts_with(&group.prefix) && command_length > group.prefix.len() + 1 {
                            built[(group.prefix.len() + 1)..].to_owned()
                        } else {
                            continue;
                        }
                    } else {
                        built.clone()
                    };

                    if let Some(ref command) = group.commands.get(&to_check) {
                        let before = self.before.clone();
                        let command = command.clone();
                        let after = self.after.clone();
                        let groups = self.groups.clone();

                        let args = if command.use_quotes() {
                            utils::parse_quotes(&message.content[position + command_length..])
                        } else {
                            message.content[position + command_length..]
                                .split_whitespace()
                                .map(|arg| arg.to_owned())
                                .collect::<Vec<String>>()
                        };

                        if let Some(error) = self.should_fail(&mut context, &message, command, args.len(), &to_check, &built) {
                            if let Some(ref handler) = self.dispatch_error_handler {
                                handler(context, message, error);
                            }
                            return;
                        }

                        thread::spawn(move || {
                            if let Some(before) = before {
                                if !(before)(&mut context, &message, &built) {
                                    return;
                                }
                            }

                            let result = command.exec(&mut context, &message, args);

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

    /// Specify the function that's called in case a command wasn't executed for one reason or another.
    ///
    /// DispatchError represents all possible fail conditions.
    ///
    /// # Examples
    ///
    /// Making a simple argument error responder:
    ///
    /// ```rust
    /// # use serenity::Client;
    /// # let mut client = Client::login("token");
    /// use serenity::framework::DispatchError::{NotEnoughArguments, TooManyArguments};
    /// 
    /// client.with_framework(|f| f
    ///     .on_dispatch_error(|ctx, msg, error| {
    ///         match error {
    ///             NotEnoughArguments { min, given } => {
    ///                 msg.channel_id.say(&format!("Need {} arguments, but only got {}.", min, given));
    ///             }
    ///             TooManyArguments { max, given } => {
    ///                 msg.channel_id.say(&format!("Max arguments allowed is {}, but got {}.", max, given));
    ///             }
    ///             _ => println!("Unhandled dispatch error.")
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
    /// ```rust
    /// # use serenity::Client;
    /// # let mut client = Client::login("token");
    /// #
    /// client.with_framework(|f| f
    ///     .before(|ctx, msg, cmd_name| {
    ///         println!("Running command {}", cmd_name);
    ///         true
    ///     }));
    /// ```
    ///
    /// Using before to prevent command usage:
    ///
    /// ```rust
    /// # use serenity::Client;
    /// # let mut client = Client::login("token");
    /// #
    /// client.with_framework(|f| f
    ///     .before(|ctx, msg, cmd_name| {
    ///         if let Ok(channel) = msg.channel_id.get() {
    ///             //  Don't run unless in nsfw channel
    ///             if !channel.is_nsfw() {
    ///                 return false;
    ///             }
    ///         }
    ///
    ///         println!("Running command {}", cmd_name);
    ///         true
    ///     }));
    /// ```
    ///
    pub fn before<F>(mut self, f: F) -> Self
        where F: Fn(&mut Context, &Message, &String) -> bool + Send + Sync + 'static {
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
    /// # use serenity::Client;
    /// # let mut client = Client::login("token");
    /// #
    /// client.with_framework(|f| f
    ///     .after(|ctx, msg, cmd_name, error| {
    ///         //  Print out an error if it happened
    ///         if let Err(why) = error {
    ///             println!("Error in {}: {:?}", cmd_name, why);
    ///         }
    ///     }));
    /// ```
    pub fn after<F>(mut self, f: F) -> Self
        where F: Fn(&mut Context, &Message, &String, Result<(), String>) + Send + Sync + 'static {
        self.after = Some(Arc::new(f));

        self
    }

    #[doc(hidden)]
    pub fn update_current_user(&mut self, user_id: UserId, is_bot: bool) {
        self.user_info = (user_id.0, is_bot);
    }
}
