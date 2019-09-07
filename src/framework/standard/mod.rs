pub mod help_commands;
pub mod macros {
    pub use command_attr::{command, group, help, check};
}

mod args;
mod configuration;
mod parse;
mod structures;

pub use args::{Args, Delimiter, Error as ArgError, Iter, RawArguments};
pub use configuration::{Configuration, WithWhiteSpace};
pub use structures::*;

use structures::buckets::{Bucket, Ratelimit};
pub use structures::buckets::BucketBuilder;

use parse::{ParseError, Invoke};
use parse::map::{CommandMap, GroupMap, Map};

use super::Framework;
use crate::client::Context;
use crate::model::{
    channel::{Channel, Message},
    permissions::Permissions,
};

use std::collections::HashMap;
use std::sync::Arc;

use threadpool::ThreadPool;
use uwl::{UnicodeStream, StrExt};

#[cfg(feature = "cache")]
use crate::cache::CacheRwLock;
#[cfg(feature = "cache")]
use crate::model::guild::{Guild, Member};
#[cfg(feature = "cache")]
use crate::internal::RwLockExt;

/// An enum representing all possible fail conditions under which a command won't
/// be executed.
#[derive(Debug)]
pub enum DispatchError {
    /// When a custom function check has failed.
    CheckFailed(&'static str, Reason),
    /// When the command requester has exceeded a ratelimit bucket. The attached
    /// value is the time a requester has to wait to run the command again.
    Ratelimited(i64),
    /// When the requested command is disabled in bot configuration.
    CommandDisabled(String),
    /// When the user is blocked in bot configuration.
    BlockedUser,
    /// When the guild or its owner is blocked in bot configuration.
    BlockedGuild,
    /// When the channel blocked in bot configuration.
    BlockedChannel,
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
    /// When the command requester lacks specific required permissions.
    LackingPermissions(Permissions),
    /// When there are too few arguments.
    NotEnoughArguments { min: u16, given: usize },
    /// When there are too many arguments.
    TooManyArguments { max: u16, given: usize },
    /// When the command was requested by a bot user when they are set to be
    /// ignored.
    IgnoredBot,
    /// When the bot ignores webhooks and a command was issued by one.
    WebhookAuthor,
    #[doc(hidden)]
    __Nonexhaustive,
}

pub type DispatchHook = dyn Fn(&mut Context, &Message, DispatchError) + Send + Sync + 'static;
type BeforeHook = dyn Fn(&mut Context, &Message, &str) -> bool + Send + Sync + 'static;
type AfterHook = dyn Fn(&mut Context, &Message, &str, Result<(), CommandError>) + Send + Sync + 'static;
type UnrecognisedHook = dyn Fn(&mut Context, &Message, &str) + Send + Sync + 'static;
type NormalMessageHook = dyn Fn(&mut Context, &Message) + Send + Sync + 'static;
type PrefixOnlyHook = dyn Fn(&mut Context, &Message) + Send + Sync + 'static;

/// A utility for easily managing dispatches to commands.
///
/// Refer to the [module-level documentation] for more information.
///
/// [module-level documentation]: index.html
#[derive(Default)]
pub struct StandardFramework {
    groups: Vec<(&'static CommandGroup, Map)>,
    buckets: HashMap<String, Bucket>,
    before: Option<Arc<BeforeHook>>,
    after: Option<Arc<AfterHook>>,
    dispatch: Option<Arc<DispatchHook>>,
    unrecognised_command: Option<Arc<UnrecognisedHook>>,
    normal_message: Option<Arc<NormalMessageHook>>,
    prefix_only: Option<Arc<PrefixOnlyHook>>,
    config: Configuration,
    help: Option<&'static HelpCommand>,
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
}

impl StandardFramework {
    #[inline]
    pub fn new() -> Self {
        StandardFramework::default()
    }

    /// Configures the framework, setting non-default values. All fields are
    /// optional. Refer to [`Configuration::default`] for more information on
    /// the default values.
    ///
    /// # Examples
    ///
    /// Configuring the framework for a [`Client`], [allowing whitespace between prefixes], and setting the [`prefix`] to `"~"`:
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
    ///         .with_whitespace(true)
    ///         .prefix("~")));
    /// ```
    ///
    /// [`Client`]: ../../client/struct.Client.html
    /// [`Configuration::default`]: struct.Configuration.html#method.default
    /// [`prefix`]: struct.Configuration.html#method.prefix
    /// [allowing whitespace between prefixes]: struct.Configuration.html#method.with_whitespace
    pub fn configure<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Configuration) -> &mut Configuration,
    {
        f(&mut self.config);

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
    /// use serenity::framework::standard::macros::command;
    /// use serenity::framework::standard::{StandardFramework, CommandResult};
    ///
    /// #[command]
    /// // Registers the bucket `basic` to this command.
    /// #[bucket = "basic"]
    /// fn nothing() -> CommandResult {
    ///     Ok(())
    /// }
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .bucket("basic", |b| b.delay(2).time_span(10).limit(3)));
    /// ```
    #[inline]
    pub fn bucket<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(&mut BucketBuilder) -> &mut BucketBuilder
    {
        let mut builder = BucketBuilder::default();

        f(&mut builder);

        let BucketBuilder {
            delay,
            time_span,
            limit,
            check,
        } = builder;

        self.buckets.insert(
            name.to_string(),
            Bucket {
                ratelimit: Ratelimit {
                    delay,
                    limit: Some((time_span, limit)),
                },
                users: HashMap::new(),
                check,
            },
        );

        self
    }

    fn should_fail_common(&self, msg: &Message) -> Option<DispatchError> {
        if self.config.ignore_bots && msg.author.bot {
            return Some(DispatchError::IgnoredBot);
        }

        if self.config.ignore_webhooks && msg.webhook_id.is_some() {
            return Some(DispatchError::WebhookAuthor);
        }

        None
    }

    fn should_fail(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
        args: &mut Args,
        command: &'static CommandOptions,
        group: &'static GroupOptions,
    ) -> Option<DispatchError> {
        if let Some(min) = command.min_args {
            if args.len() < min as usize {
                return Some(DispatchError::NotEnoughArguments {
                    min,
                    given: args.len(),
                });
            }
        }

        if let Some(max) = command.max_args {
            if args.len() > max as usize {
                return Some(DispatchError::TooManyArguments {
                    max,
                    given: args.len(),
                });
            }
        }

        if (group.owner_privilege && command.owner_privilege)
            && self.config.owners.contains(&msg.author.id)
        {
            return None;
        }

        if self.config.blocked_users.contains(&msg.author.id) {
            return Some(DispatchError::BlockedUser);
        }

        #[cfg(feature = "cache")]
        {
            if let Some(Channel::Guild(chan)) = msg.channel_id.to_channel_cached(&ctx.cache) {
                let guild_id = chan.with(|c| c.guild_id);

                if self.config.blocked_guilds.contains(&guild_id) {
                    return Some(DispatchError::BlockedGuild);
                }

                if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
                    if self.config.blocked_users.contains(&guild.with(|g| g.owner_id)) {
                        return Some(DispatchError::BlockedGuild);
                    }
                }
            }
        }

        if !self.config.allowed_channels.is_empty() &&
           !self.config.allowed_channels.contains(&msg.channel_id) {
            return Some(DispatchError::BlockedChannel);
        }

        if let Some(ref mut bucket) = command.bucket.as_ref().and_then(|b| self.buckets.get_mut(*b)) {
            let rate_limit = bucket.take(msg.author.id.0);

            let apply = bucket.check.as_ref().map_or(true, |check| {
                (check)(ctx, msg.guild_id, msg.channel_id, msg.author.id)
            });

            if apply && rate_limit > 0 {
                return Some(DispatchError::Ratelimited(rate_limit));
            }
        }

        for check in group.checks.iter().chain(command.checks.iter()) {
            let res = (check.function)(ctx, msg, args, command);

            if let CheckResult::Failure(r) = res {
                return Some(DispatchError::CheckFailed(check.name, r));
            }
        }

        None
    }

    /// Adds a group which can organize several related commands.
    /// Groups are taken into account when using
    /// `serenity::framework::standard::help_commands`.
    ///
    /// # Examples
    ///
    /// Add a group with ping and pong commands:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use std::error::Error as StdError;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// #
    /// use serenity::client::{Client, Context};
    /// use serenity::model::channel::Message;
    /// use serenity::framework::standard::{
    ///     StandardFramework,
    ///     CommandResult,
    ///     macros::{command, group},
    /// };
    ///
    /// // For information regarding this macro, learn more about it in its documentation in `command_attr`.
    /// #[command]
    /// fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    ///     msg.channel_id.say(&ctx.http, "pong!")?;
    ///
    ///     Ok(())
    /// }
    ///
    /// #[command]
    /// fn pong(ctx: &mut Context, msg: &Message) -> CommandResult {
    ///     msg.channel_id.say(&ctx.http, "ping!")?;
    ///
    ///     Ok(())
    /// }
    ///
    /// #[group("bingbong")]
    /// #[commands(ping, pong)]
    /// struct BingBong;
    ///
    /// # fn main() -> Result<(), Box<dyn StdError>> {
    /// #   let mut client = Client::new("token", Handler)?;
    /// client.with_framework(StandardFramework::new()
    ///     // Groups' names are changed to all uppercase, plus appended with `_GROUP`.
    ///     .group(&BINGBONG_GROUP));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn group(mut self, group: &'static CommandGroup) -> Self {
        self.group_add(group);
        self.initialized = true;

        self
    }

    /// Adds a group to be used by the framework. Primary use-case is runtime modification
    /// of groups in the framework; will _not_ mark the framework as initialized. Refer to
    /// [`group`] for adding groups in initial configuration.
    ///
    /// Note: does _not_ return `Self` like many other commands. This is because
    /// it's not intended to be chained as the other commands are.
    ///
    /// [`group`]: #method.group
    pub fn group_add(&mut self, group: &'static CommandGroup) {
        let map = if group.options.prefixes.is_empty() {
            Map::Prefixless(GroupMap::new(&group.options.sub_groups), CommandMap::new(&group.options.commands))
        } else {
            Map::WithPrefixes(GroupMap::new(&[group]))
        };

        self.groups.push((group, map));
    }

    /// Removes a group from being used in the framework. Primary use-case is runtime modification
    /// of groups in the framework.
    ///
    /// Note: does _not_ return `Self` like many other commands. This is because
    /// it's not intended to be chained as the other commands are.
    pub fn group_remove(&mut self, group: &'static CommandGroup) {
        // Iterates through the vector and if a given group _doesn't_ match, we retain it
        self.groups.retain(|&(g, _)| g != group)
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
    ///     .on_dispatch_error(|context, msg, error| {
    ///         match error {
    ///             NotEnoughArguments { min, given } => {
    ///                 let s = format!("Need {} arguments, but only got {}.", min, given);
    ///
    ///                 let _ = msg.channel_id.say(&context.http, &s);
    ///             },
    ///             TooManyArguments { max, given } => {
    ///                 let s = format!("Max arguments allowed is {}, but got {}.", max, given);
    ///
    ///                 let _ = msg.channel_id.say(&context.http, &s);
    ///             },
    ///             _ => println!("Unhandled dispatch error."),
    ///         }
    ///     }));
    /// ```
    pub fn on_dispatch_error<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context, &Message, DispatchError) + Send + Sync + 'static,
    {
        self.dispatch = Some(Arc::new(f));

        self
    }

    /// Specify the function to be called on messages comprised of only the prefix.
    pub fn prefix_only<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context, &Message) + Send + Sync + 'static
    {
        self.prefix_only = Some(Arc::new(f));

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
    ///     .before(|ctx, msg, cmd_name| {
    ///         if let Ok(channel) = msg.channel_id.to_channel(ctx) {
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
    where
        F: Fn(&mut Context, &Message, &str) -> bool + Send + Sync + 'static,
    {
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
    where
        F: Fn(&mut Context, &Message, &str, Result<(), CommandError>) + Send + Sync + 'static,
    {
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
    ///     .unrecognised_command(|_ctx, msg, unrecognised_command_name| {
    ///        println!("A user named {:?} tried to executute an unknown command: {}", msg.author.name, unrecognised_command_name);
    ///     }));
    /// ```
    pub fn unrecognised_command<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context, &Message, &str) + Send + Sync + 'static,
    {
        self.unrecognised_command = Some(Arc::new(f));

        self
    }

    /// Specify the function to be called if a message contains no command.
    ///
    /// # Examples
    ///
    /// Using `normal_message`:
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
    ///     .normal_message(|ctx, msg| {
    ///         println!("Received a generic message: {:?}", msg.content);
    ///     }));
    /// ```
    pub fn normal_message<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context, &Message) + Send + Sync + 'static,
    {
        self.normal_message = Some(Arc::new(f));

        self
    }

    /// Sets what code should be executed when a user sends `(prefix)help`.
    ///
    /// If a [`command`] named `help` in a group was set, then this takes precedence first.
    ///
    /// [`command`]: #method.command
    pub fn help(mut self, h: &'static HelpCommand) -> Self {
        self.help = Some(h);

        self
    }
}

impl Framework for StandardFramework {
    fn dispatch(&mut self, mut ctx: Context, msg: Message, threadpool: &ThreadPool) {
        let mut stream = UnicodeStream::new(&msg.content);

        stream.take_while(|s| s.is_whitespace());

        let prefix = parse::prefix(&mut ctx, &msg, &mut stream, &self.config);

        if prefix.is_some() && stream.rest().is_empty() {

            if let Some(prefix_only) = &self.prefix_only {
                let prefix_only = Arc::clone(&prefix_only);
                let msg = msg.clone();

                threadpool.execute(move || {
                    prefix_only(&mut ctx, &msg);
                });
            }

            return;
        }

        if prefix.is_none() && !(self.config.no_dm_prefix && msg.is_private()) {

            if let Some(normal) = &self.normal_message {
                let normal = Arc::clone(&normal);
                let msg = msg.clone();

                threadpool.execute(move || {
                    normal(&mut ctx, &msg);
                });
            }

            return;
        }

        if let Some(error) = self.should_fail_common(&msg) {

            if let Some(dispatch) = &self.dispatch {
                dispatch(&mut ctx, &msg, error);
            }

            return;
        }

        let invocation = parse::command(
            &ctx,
            &msg,
            &mut stream,
            &self.groups,
            &self.config,
            self.help.as_ref().map(|h| h.options.names),
        );

        let invoke = match invocation {
            Ok(i) => i,
            Err(ParseError::UnrecognisedCommand(unreg)) => {
                if let Some(unreg) = unreg {
                    if let Some(unrecognised_command) = &self.unrecognised_command {
                        let unrecognised_command = Arc::clone(&unrecognised_command);
                        let mut ctx = ctx.clone();
                        let msg = msg.clone();
                        threadpool.execute(move || {
                            unrecognised_command(&mut ctx, &msg, &unreg);
                        });
                    }
                }

                if let Some(normal) = &self.normal_message {
                    let normal = Arc::clone(&normal);
                    let msg = msg.clone();

                    threadpool.execute(move || {
                        normal(&mut ctx, &msg);
                    });
                }

                return;
            }
            Err(ParseError::Dispatch(error)) => {
                if let Some(dispatch) = &self.dispatch {
                    dispatch(&mut ctx, &msg, error);
                }

                return;
            }
        };

        match invoke {
            Invoke::Help(name) => {
                let args = Args::new(stream.rest(), &self.config.delimiters);

                let before = self.before.clone();
                let after = self.after.clone();
                let owners = self.config.owners.clone();

                let groups = self.groups.iter().map(|(g, _)| *g).collect::<Vec<_>>();

                let msg = msg.clone();

                // `parse_command` promises to never return a help invocation if `StandardFramework::help` is `None`.
                let help = self.help.unwrap();

                threadpool.execute(move || {
                    if let Some(before) = before {
                        if !before(&mut ctx, &msg, name) {
                            return;
                        }
                    }

                    let res = (help.fun)(&mut ctx, &msg, args, help.options, &groups, owners);

                    if let Some(after) = after {
                        after(&mut ctx, &msg, name, res);
                    }
                });
            }
            Invoke::Command { command, group } => {
                let mut args = {
                    use std::borrow::Cow;

                    let mut delims = Cow::Borrowed(&self.config.delimiters);

                    // If user has configured the command's own delimiters, use those instead.
                    if !command.options.delimiters.is_empty() {
                        // FIXME: Get rid of this allocation.
                        let mut v = Vec::with_capacity(command.options.delimiters.len());

                        for delim in command.options.delimiters {
                            if delim.len() == 1 {
                                v.push(Delimiter::Single(delim.chars().next().unwrap()));
                            } else {
                                // This too.
                                v.push(Delimiter::Multiple(delim.to_string()));
                            }
                        }

                        delims = Cow::Owned(v);
                    }

                    Args::new(stream.rest(), &delims)
                };

                if let Some(error) =
                    self.should_fail(&mut ctx, &msg, &mut args, &command.options, &group.options)
                {
                    if let Some(dispatch) = &self.dispatch {
                        dispatch(&mut ctx, &msg, error);
                    }

                    return;
                }

                let before = self.before.clone();
                let after = self.after.clone();
                let msg = msg.clone();
                let name = &command.options.names[0];
                threadpool.execute(move || {
                    if let Some(before) = before {
                        if !before(&mut ctx, &msg, name) {
                            return;
                        }
                    }

                    let res = (command.fun)(&mut ctx, &msg, args);

                    if let Some(after) = after {
                        after(&mut ctx, &msg, name, res);
                    }
                });
            }
        }
    }
}

pub trait CommonOptions {
    fn required_permissions(&self) -> &Permissions;
    fn allowed_roles(&self) -> &'static [&'static str];
    fn only_in(&self) -> OnlyIn;
    fn help_available(&self) -> bool;
    fn owners_only(&self) -> bool;
    fn owner_privilege(&self) -> bool;
}

impl CommonOptions for &GroupOptions {
    fn required_permissions(&self) -> &Permissions {
        &self.required_permissions
    }

    fn allowed_roles(&self) -> &'static [&'static str] {
        &self.allowed_roles
    }

    fn only_in(&self) -> OnlyIn {
        self.only_in
    }

    fn help_available(&self) -> bool {
        self.help_available
    }

    fn owners_only(&self) -> bool {
        self.owners_only
    }

    fn owner_privilege(&self) -> bool {
        self.owner_privilege
    }
}

impl CommonOptions for &CommandOptions {
    fn required_permissions(&self) -> &Permissions {
        &self.required_permissions
    }

    fn allowed_roles(&self) -> &'static [&'static str] {
        &self.allowed_roles
    }

    fn only_in(&self) -> OnlyIn {
        self.only_in
    }

    fn help_available(&self) -> bool {
        self.help_available
    }

    fn owners_only(&self) -> bool {
        self.owners_only
    }

    fn owner_privilege(&self) -> bool {
        self.owner_privilege
    }
}

#[cfg(feature = "cache")]
pub(crate) fn has_correct_permissions(
    cache: impl AsRef<CacheRwLock>,
    options: &impl CommonOptions,
    message: &Message,
) -> bool {
    if options.required_permissions().is_empty() {
        true
    } else if let Some(guild) = message.guild(&cache) {
        let perms = guild.with(|g| g.user_permissions_in(message.channel_id, message.author.id));

        perms.contains(*options.required_permissions())
    } else {
        false
    }
}

#[cfg(all(feature = "cache", feature = "http"))]
pub(crate) fn has_correct_roles(
    options: &impl CommonOptions,
    guild: &Guild,
    member: &Member)
-> bool {
    if options.allowed_roles().is_empty() {
        true
    } else {
        options.allowed_roles()
            .iter()
            .flat_map(|r| guild.role_by_name(r))
            .any(|g| member.roles.contains(&g.id))
    }
}
