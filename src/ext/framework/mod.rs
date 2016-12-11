//! The framework is a customizable method of separating commands, used in
//! combination with [`Client::with_framework`].
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
//! ```rust,no_run
//! use serenity::client::{Client, Context};
//! use serenity::model::Message;
//! use std::env;
//!
//! let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN").unwrap());
//!
//! client.with_framework(|f| f
//!     .configure(|c| c.prefix("~"))
//!     .command("about", |c| c.exec_str("A simple test bot"))
//!     .command("ping", |c| c.exec(ping)));
//!
//! fn about(context: &Context, _message: &Message, _args: Vec<String>) {
//!     let _ = context.say("A simple test bot");
//! }
//!
//! fn ping(context: &Context, _message: &Message, _args: Vec<String>) {
//!     let _ = context.say("Pong!");
//! }
//! ```
//!
//! [`Client::with_framework`]: ../../client/struct.Client.html#method.with_framework

mod command;
mod configuration;
mod create_command;

pub use self::command::{Command, CommandType, CommandGroup};
pub use self::configuration::{AccountType, Configuration};
pub use self::create_command::CreateCommand;

use self::command::{Hook, InternalCommand};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use ::client::Context;
use ::model::Message;
use ::utils;
use ::client::CACHE;
use ::model::Permissions;

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
/// command!(multiply(_context, message, _args, first: f64, second: f64) {
///     let product = first * second;
///
///     if let Err(why) = message.reply(&product.to_string()) {
///         println!("Error sending product: {:?}", why);
///     }
/// });
/// ```
///
/// [`Framework`]: ext/framework/index.html
#[macro_export]
macro_rules! command {
    ($fname:ident($c:ident, $m:ident, $a:ident) $b:block) => {
        pub fn $fname($c: &Context, $m: &Message, $a: Vec<String>) {
            $b
        }
    };
    ($fname:ident($c:ident, $m:ident, $a:ident, $($name:ident: $t:ty),*) $b:block) => {
        pub fn $fname($c: &Context, $m: &Message, $a: Vec<String>) {
            let mut i = $a.iter();

            $(
                let $name = match i.next() {
                    Some(v) => match v.parse::<$t>() {
                        Ok(v) => v,
                        Err(_why) => return,
                    },
                    None => return,
                };
            )*

            drop(i);

            $b
        }
    };
}

/// A utility for easily managing dispatches to commands.
///
/// Refer to the [module-level documentation] for more information.
///
/// [module-level documentation]: index.html
#[allow(type_complexity)]
#[derive(Default)]
pub struct Framework {
    configuration: Configuration,
    commands: HashMap<String, InternalCommand>,
    groups: HashMap<String, Arc<CommandGroup>>,
    before: Option<Arc<Hook>>,
    after: Option<Arc<Hook>>,
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
    /// [`Client::on_message`]: ../../client/struct.Client.html#method.on_message
    /// [`Event::MessageCreate`]: ../../model/event/enum.Event.html#variant.MessageCreate
    pub initialized: bool,
}

impl Framework {
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
    /// let mut client = Client::login_bot(&env::var("DISCORD_TOKEN").unwrap());
    /// client.with_framework(|f| f
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

    #[doc(hidden)]
    pub fn dispatch(&mut self, context: Context, message: Message) {
        match self.configuration.account_type {
            AccountType::Selfbot => {
                if message.author.id != CACHE.read().unwrap().user.id {
                    return;
                }
            },
            AccountType::Bot => {
                if message.author.bot {
                    return;
                }
            },
            AccountType::Automatic => {
                let cache = CACHE.read().unwrap();
                if cache.user.bot {
                    if message.author.bot {
                        return;
                    }
                } else if message.author.id != cache.user.id {
                    return;
                }
            },
            AccountType::Any => {}
        }
        let res = command::positions(&context, &message.content, &self.configuration);

        let positions = match res {
            Some(positions) => positions,
            None => return,
        };

        // Ensure that the message length is at least longer than a prefix
        // length. There's no point in checking further ahead if there's nothing
        // _to_ check.
        if positions.iter().all(|p| message.content.len() <= *p) {
            return;
        }

        'outer: for position in positions {
            let mut built = String::new();

            for i in 0..self.configuration.depth {
                if i != 0 {
                    built.push(' ');
                }

                built.push_str(match {
                    message.content
                        .split_at(position)
                        .1
                        .trim()
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .get(i)
                } {
                    Some(piece) => piece,
                    None => continue,
                });
                for (_, group) in &self.groups {
                    let to_check = if let Some(ref prefix) = group.prefix {
                        if built.starts_with(prefix) {
                            built[prefix.len()..].to_owned()
                        } else {
                            continue;
                        }
                    } else {
                        built.clone()
                    };

                    if let Some(command) = self.commands.get(&to_check) {
                        if message.is_private() {
                            if command.guild_only {
                                return;
                            }
                        } else if command.dm_only {
                            return;
                        }

                        for check in &command.checks {
                            if !(check)(&context, &message) {
                                continue 'outer;
                            }
                        }

                        let before = self.before.clone();
                        let command = command.clone();
                        let after = self.after.clone();
                        let groups = self.groups.clone();

                        thread::spawn(move || {
                            if let Some(before) = before {
                                (before)(&context, &message, &built);
                            }

                            let args = if command.use_quotes {
                                utils::parse_quotes(&message.content[position + built.len()..])
                            } else {
                                message.content[position + built.len()..]
                                    .split_whitespace()
                                    .map(|arg| arg.to_owned())
                                    .collect::<Vec<String>>()
                            };

                            if let Some(x) = command.min_args {
                                if args.len() < x as usize {
                                    return;
                                }
                            }

                            if let Some(x) = command.max_args {
                                if args.len() > x as usize {
                                    return;
                                }
                            }

                            if !command.required_permissions.is_empty() {
                                let mut permissions_fulfilled = false;

                                if let Some(member) = message.get_member() {
                                    let cache = CACHE.read().unwrap();

                                    if let Ok(guild_id) = member.find_guild() {
                                        if let Some(guild) = cache.get_guild(guild_id) {
                                            let perms = guild.permissions_for(message.channel_id, message.author.id);

                                            permissions_fulfilled = perms.contains(command.required_permissions);
                                        }
                                    }
                                }

                                if !permissions_fulfilled {
                                    return;
                                }
                            }

                            match command.exec {
                                CommandType::StringResponse(ref x) => {
                                    let _ = &context.say(x);
                                },
                                CommandType::Basic(ref x) => {
                                    (x)(&context, &message, args);
                                },
                                CommandType::WithCommands(ref x) => {
                                    (x)(&context, &message, groups, args);
                                }
                            }

                            if let Some(after) = after {
                                (after)(&context, &message, &built);
                            }
                        });

                        return;
                    }
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
    /// Note that once v0.2.0 lands, you will need to use the command builder
    /// via the [`command`] method to set checks. This command will otherwise
    /// only be for simple commands.
    ///
    /// Refer to the [module-level documentation] for more information and
    /// usage.
    ///
    /// [`command`]: #method.command
    /// [module-level documentation]: index.html
    pub fn on<F, S>(mut self, command_name: S, f: F) -> Self
        where F: Fn(&Context, &Message, Vec<String>) + Send + Sync + 'static,
              S: Into<String> {
        if !self.groups.contains_key("Ungrouped") {
            self.groups.insert("Ungrouped".to_string(), Arc::new(CommandGroup {
                prefix: None,
                commands: HashMap::new()
            }));
        }

        if let Some(ref mut x) = self.groups.get_mut("Ungrouped") {
            x.commands.insert(command_name.into(), Command {
                checks: Vec::default(),
                exec: CommandType::Basic(Box::new(f)),
                desc: None,
                usage: None,
                use_quotes: false,
                dm_only: false,
                guild_only: false,
                help_available: true,
                min_args: None,
                max_args: None,
                required_permissions: Permissions::empty()
            });
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
        where F: FnOnce(CreateCommand) -> CreateCommand,
              S: Into<String> {
        let cmd = f(CreateCommand(Command::default())).0;
        if !self.groups.contains_key("Ungrouped") {
            self.groups.insert("Ungrouped".to_string(), Arc::new(CommandGroup {
                prefix: None,
                commands: HashMap::new()
            }));
        }

        if let Some(ref mut x) = self.groups.get_mut("Ungrouped") {
            x.commands.insert(command_name.into(), cmd);
        }

        self.initialized = true;

        self
    }

    /// Specify the function to be called prior to every command's execution.
    pub fn before<F>(mut self, f: F) -> Self
        where F: Fn(&Context, &Message, &String) + Send + Sync + 'static {
        self.before = Some(Arc::new(f));

        self
    }

    /// Specify the function to be called after every command's execution.
    pub fn after<F>(mut self, f: F) -> Self
        where F: Fn(&Context, &Message, &String) + Send + Sync + 'static {
        self.after = Some(Arc::new(f));

        self
    }
}
