pub mod help_commands;
pub mod macros {
    pub use command_attr::{command, group, help, check, hook};
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
    channel::Message,
    permissions::Permissions,
};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use futures::future::BoxFuture;
use uwl::Stream;
use async_trait::async_trait;
use tracing::instrument;

#[cfg(feature = "cache")]
use crate::model::channel::Channel;
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "cache")]
use crate::model::guild::Member;
#[cfg(all(feature = "cache", feature = "http", feature = "model"))]
use crate::model::{guild::Role, id::RoleId};

/// An enum representing all possible fail conditions under which a command won't
/// be executed.
#[derive(Debug)]
#[non_exhaustive]
pub enum DispatchError {
    /// When a custom function check has failed.
    CheckFailed(&'static str, Reason),
    /// When the command requester has exceeded a ratelimit bucket. The attached
    /// value is the time a requester has to wait to run the command again.
    Ratelimited(Duration),
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
}

type DispatchHook = for<'fut> fn(&'fut Context, &'fut Message, DispatchError) -> BoxFuture<'fut , ()>;
type BeforeHook = for<'fut> fn(&'fut Context, &'fut Message, &'fut str) -> BoxFuture<'fut, bool>;
type AfterHook = for<'fut> fn(&'fut Context, &'fut Message, &'fut str, Result<(), CommandError>) -> BoxFuture<'fut, ()>;
type UnrecognisedHook = for<'fut> fn(&'fut Context, &'fut Message, &'fut str) -> BoxFuture<'fut, ()>;
type NormalMessageHook = for<'fut> fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, ()>;
type PrefixOnlyHook = for<'fut> fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, ()>;

/// A utility for easily managing dispatches to commands.
///
/// Refer to the [module-level documentation] for more information.
///
/// [module-level documentation]: index.html
#[derive(Default)]
pub struct StandardFramework {
    groups: Vec<(&'static CommandGroup, Map)>,
    buckets: Mutex<HashMap<String, Bucket>>,
    before: Option<BeforeHook>,
    after: Option<AfterHook>,
    dispatch: Option<DispatchHook>,
    unrecognised_command: Option<UnrecognisedHook>,
    normal_message: Option<NormalMessageHook>,
    prefix_only: Option<PrefixOnlyHook>,
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
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let token = std::env::var("DISCORD_TOKEN")?;
    /// let framework = StandardFramework::new()
    ///     .configure(|c| c
    ///         .with_whitespace(true)
    ///         .prefix("~"));
    ///
    /// let mut client = Client::builder(&token).event_handler(Handler).framework(framework).await?;
    /// #     Ok(())
    /// # }
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
    /// use serenity::framework::standard::macros::command;
    /// use serenity::framework::standard::{StandardFramework, CommandResult};
    ///
    /// #[command]
    /// // Registers the bucket `basic` to this command.
    /// #[bucket = "basic"]
    /// async fn nothing() -> CommandResult {
    ///     Ok(())
    /// }
    ///
    /// # async fn run() {
    /// let framework = StandardFramework::new()
    ///     .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
    ///     .await;
    /// # }
    /// ```
    #[inline]
    pub async fn bucket<F>(self, name: &str, f: F) -> Self
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

        self.buckets.lock().await.insert(
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

    /// Whether the message should be ignored because it is from a bot or webhook.
    fn should_ignore(&self, msg: &Message) -> bool {
        (self.config.ignore_bots && msg.author.bot) ||
            (self.config.ignore_webhooks && msg.webhook_id.is_some())
    }

    async fn should_fail<'a>(
        &'a self,
        ctx: &'a Context,
        msg: &'a Message,
        args: &'a mut Args,
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
            if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx).await {
                let guild_id = channel.guild_id;

                if self.config.blocked_guilds.contains(&guild_id) {
                    return Some(DispatchError::BlockedGuild);
                }

                if let Some(guild) = guild_id.to_guild_cached(&ctx.cache).await {
                    if self.config.blocked_users.contains(&guild.owner_id) {
                        return Some(DispatchError::BlockedGuild);
                    }
                }
            }
        }

        if !self.config.allowed_channels.is_empty() &&
           !self.config.allowed_channels.contains(&msg.channel_id) {
            return Some(DispatchError::BlockedChannel);
        }

        {
            let mut buckets = self.buckets.lock().await;

            if let Some(ref mut bucket) = command.bucket.as_ref().and_then(|b| buckets.get_mut(*b)) {
                let rate_limit = bucket.take(msg.author.id.0);

                let apply = match bucket.check.as_ref() {
                    Some(check) => (check)(ctx, msg.guild_id, msg.channel_id, msg.author.id).await,
                    None => true,
                };

                if let Some(rate_limit)= rate_limit {
                    if apply {
                        return Some(DispatchError::Ratelimited(rate_limit))
                    }
                }
            }
        }

        for check in group.checks.iter().chain(command.checks.iter()) {
            let res = (check.function)(ctx, msg, args, command).await;

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
    /// async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    ///     msg.channel_id.say(&ctx.http, "pong!").await?;
    ///
    ///     Ok(())
    /// }
    ///
    /// #[command]
    /// async fn pong(ctx: &Context, msg: &Message) -> CommandResult {
    ///     msg.channel_id.say(&ctx.http, "ping!").await?;
    ///
    ///     Ok(())
    /// }
    ///
    /// #[group("bingbong")]
    /// #[commands(ping, pong)]
    /// struct BingBong;
    ///
    /// let framework = StandardFramework::new()
    ///     // Groups' names are changed to all uppercase, plus appended with `_GROUP`.
    ///     .group(&BINGBONG_GROUP);
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
            Map::Prefixless(
                GroupMap::new(&group.options.sub_groups, &self.config),
                CommandMap::new(&group.options.commands, &self.config),
            )
        } else {
            Map::WithPrefixes(GroupMap::new(&[group], &self.config))
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
    /// # use serenity::model::prelude::*;
    /// use serenity::framework::standard::macros::hook;
    /// use serenity::framework::standard::DispatchError;
    /// use serenity::framework::StandardFramework;
    ///
    /// #[hook]
    /// async fn dispatch_error_hook(context: &Context, msg: &Message, error: DispatchError) {
    ///     match error {
    ///         DispatchError::NotEnoughArguments { min, given } => {
    ///             let s = format!("Need {} arguments, but only got {}.", min, given);
    ///
    ///             let _ = msg.channel_id.say(&context, &s).await;
    ///         },
    ///         DispatchError::TooManyArguments { max, given } => {
    ///             let s = format!("Max arguments allowed is {}, but got {}.", max, given);
    ///
    ///             let _ = msg.channel_id.say(&context, &s).await;
    ///         },
    ///         _ => println!("Unhandled dispatch error."),
    ///     }
    /// }
    ///
    /// let framework = StandardFramework::new()
    ///     .on_dispatch_error(dispatch_error_hook);
    /// ```
    pub fn on_dispatch_error(mut self, f: DispatchHook) -> Self {
        self.dispatch = Some(f);

        self
    }

    /// Specify the function to be called on messages comprised of only the prefix.
    pub fn prefix_only(mut self, f: PrefixOnlyHook) -> Self {
        self.prefix_only = Some(f);

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
    /// # use serenity::model::prelude::*;
    /// use serenity::framework::standard::macros::hook;
    /// use serenity::framework::StandardFramework;
    ///
    /// #[hook]
    /// async fn before_hook(_: &Context, _: &Message, cmd_name: &str) -> bool {
    ///     println!("Running command {}", cmd_name);
    ///     true
    /// }
    /// let framework = StandardFramework::new()
    ///     .before(before_hook);
    /// ```
    ///
    /// Using before to prevent command usage:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::prelude::*;
    /// use serenity::framework::standard::macros::hook;
    /// use serenity::framework::StandardFramework;
    ///
    /// #[hook]
    /// async fn before_hook(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    ///     if let Ok(channel) = msg.channel_id.to_channel(ctx).await {
    ///         //  Don't run unless in nsfw channel
    ///         if !channel.is_nsfw() {
    ///             return false;
    ///         }
    ///     }
    ///
    ///     println!("Running command {}", cmd_name);
    ///
    ///     true
    /// }
    ///
    /// let framework = StandardFramework::new()
    ///     .before(before_hook);
    /// ```
    pub fn before(mut self, f: BeforeHook) -> Self {
        self.before = Some(f);

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
    /// # use serenity::model::prelude::*;
    /// use serenity::framework::standard::macros::hook;
    /// use serenity::framework::standard::CommandError;
    /// use serenity::framework::StandardFramework;
    ///
    /// #[hook]
    /// async fn after_hook(_: &Context, _: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    ///     //  Print out an error if it happened
    ///     if let Err(why) = error {
    ///         println!("Error in {}: {:?}", cmd_name, why);
    ///     }
    /// }
    ///
    /// let framework = StandardFramework::new()
    ///     .after(after_hook);
    /// ```
    pub fn after(mut self, f: AfterHook) -> Self {
        self.after = Some(f);

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
    /// # use serenity::model::prelude::*;
    /// use serenity::framework::standard::macros::hook;
    /// use serenity::framework::StandardFramework;
    ///
    /// #[hook]
    /// async fn unrecognised_command_hook(_: &Context, msg: &Message, unrecognised_command_name: &str) {
    ///     println!("A user named {:?} tried to executute an unknown command: {}",
    ///         msg.author.name, unrecognised_command_name
    ///     );
    /// }
    ///
    /// let framework = StandardFramework::new()
    ///     .unrecognised_command(unrecognised_command_hook);
    /// ```
    pub fn unrecognised_command(mut self, f: UnrecognisedHook) -> Self {
        self.unrecognised_command = Some(f);

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
    /// # use serenity::model::prelude::*;
    /// use serenity::framework::standard::macros::hook;
    /// use serenity::framework::StandardFramework;
    ///
    /// #[hook]
    /// async fn normal_message_hook(_: &Context, msg: &Message) {
    ///     println!("Received a generic message: {:?}", msg.content);
    /// }
    ///
    /// let framework = StandardFramework::new()
    ///     .normal_message(normal_message_hook);
    /// ```
    pub fn normal_message(mut self, f: NormalMessageHook) -> Self {
        self.normal_message = Some(f);

        self
    }

    /// Sets what code should be executed when a user sends `(prefix)help`.
    ///
    /// If a command named `help` in a group was set, then this takes precedence first.
    pub fn help(mut self, h: &'static HelpCommand) -> Self {
        self.help = Some(h);

        self
    }
}

#[async_trait]
impl Framework for StandardFramework {
    #[instrument(skip(self, ctx))]
    async fn dispatch(&self, mut ctx: Context, msg: Message) {
        if self.should_ignore(&msg) {
            return;
        }

        let mut stream = Stream::new(&msg.content);

        stream.take_while_char(|c| c.is_whitespace());

        let prefix = parse::prefix(&ctx, &msg, &mut stream, &self.config).await;

        if prefix.is_some() && stream.rest().is_empty() {
            if let Some(prefix_only) = &self.prefix_only {
                prefix_only(&mut ctx, &msg).await;
            }

            return;
        }

        if prefix.is_none() && !(self.config.no_dm_prefix && msg.is_private()) {
            if let Some(normal) = &self.normal_message {
                normal(&mut ctx, &msg).await;
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
        ).await;

        let invoke = match invocation {
            Ok(i) => i,
            Err(ParseError::UnrecognisedCommand(unreg)) => {
                if let Some(unreg) = unreg {
                    if let Some(unrecognised_command) = &self.unrecognised_command {
                        unrecognised_command(&mut ctx, &msg, &unreg).await;
                    }
                }

                if let Some(normal) = &self.normal_message {
                    normal(&mut ctx, &msg).await;
                }

                return;
            }
            Err(ParseError::Dispatch(error)) => {
                if let Some(dispatch) = &self.dispatch {
                    dispatch(&mut ctx, &msg, error).await;
                }

                return;
            }
        };

        match invoke {
            Invoke::Help(name) => {
                let args = Args::new(stream.rest(), &self.config.delimiters);

                let owners = self.config.owners.clone();
                let groups = self.groups.iter().map(|(g, _)| *g).collect::<Vec<_>>();

                // `parse_command` promises to never return a help invocation if `StandardFramework::help` is `None`.
                let help = self.help.unwrap();

                if let Some(before) = &self.before {
                    if !before(&mut ctx, &msg, name).await {
                        return;
                    }
                }

                let res = (help.fun)(&mut ctx, &msg, args, help.options, &groups, owners).await;

                if let Some(after) = &self.after {
                    after(&mut ctx, &msg, name, res).await;
                }
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
                    self.should_fail(&ctx, &msg, &mut args, &command.options, &group.options).await
                {
                    if let Some(dispatch) = &self.dispatch {
                        dispatch(&mut ctx, &msg, error).await;
                    }

                    return;
                }

                let name = command.options.names[0];

                if let Some(before) = &self.before {
                    if !before(&mut ctx, &msg, name).await {
                        return;
                    }
                }

                let res = (command.fun)(&mut ctx, &msg, args).await;

                if let Some(after) = &self.after {
                    after(&mut ctx, &msg, name, res).await;
                }
            }
        }
    }
}

pub trait CommonOptions {
    fn required_permissions(&self) -> &Permissions;
    fn allowed_roles(&self) -> &'static [&'static str];
    fn checks(&self) -> &'static [&'static Check];
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

    fn checks(&self) -> &'static [&'static Check] {
        &self.checks
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

    fn checks(&self) -> &'static [&'static Check] {
        &self.checks
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
pub(crate) async fn has_correct_permissions(
    cache: impl AsRef<Cache>,
    options: &impl CommonOptions,
    message: &Message,
) -> bool {
    if options.required_permissions().is_empty() {
        true
    } else if let Some(guild) = message.guild(&cache).await {
        let perms = guild.user_permissions_in(message.channel_id, message.author.id);

        perms.contains(*options.required_permissions())
    } else {
        false
    }
}

#[cfg(all(feature = "cache", feature = "http"))]
pub(crate) fn has_correct_roles(
    options: &impl CommonOptions,
    roles: &HashMap<RoleId, Role>,
    member: &Member)
-> bool {
    if options.allowed_roles().is_empty() {
        true
    } else {
        options.allowed_roles()
            .iter()
            .flat_map(|r| roles.values().find(|role| *r == role.name))
            .any(|g| member.roles.contains(&g.id))
    }
}
