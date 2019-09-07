use super::Delimiter;
use crate::client::Context;
use crate::model::{channel::Message, id::{UserId, GuildId, ChannelId}};
use std::collections::HashSet;

type DynamicPrefixHook = dyn Fn(&mut Context, &Message) -> Option<String> + Send + Sync + 'static;

/// A configuration struct for deciding whether the framework
/// should allow optional whitespace between prefixes, group prefixes and command names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WithWhiteSpace {
    pub prefixes: bool,
    pub groups: bool,
    pub commands: bool,
}

impl Default for WithWhiteSpace {
    /// Impose the default settings to (false, true, true).
    fn default() -> Self {
        WithWhiteSpace {
            prefixes: false,
            groups: true,
            commands: true,
        }
    }
}

impl From<bool> for WithWhiteSpace {
    /// Impose the prefix setting.
    fn from(b: bool) -> Self {
        // Assume that they want to do this for prefixes
        WithWhiteSpace {
            prefixes: b,
            ..Default::default()
        }
    }
}

impl From<(bool, bool)> for WithWhiteSpace {
    /// Impose the prefix and group prefix settings.
    fn from((prefixes, groups): (bool, bool)) -> Self {
        WithWhiteSpace {
            prefixes,
            groups,
            ..Default::default()
        }
    }
}

impl From<(bool, bool, bool)> for WithWhiteSpace {
    /// Impose the prefix, group prefix and command names settings.
    fn from((prefixes, groups, commands): (bool, bool, bool)) -> Self {
        WithWhiteSpace {
            prefixes,
            groups,
            commands,
        }
    }
}

/// The configuration to use for a [`StandardFramework`] associated with a [`Client`]
/// instance.
///
/// This allows setting configurations like the depth to search for commands,
/// whether to treat mentions like a command prefix, etc.
///
/// To see the default values, refer to the [default implementation].
///
/// # Examples
///
/// Responding to mentions and setting a command prefix of `"~"`:
///
/// ```rust,no_run
/// # use serenity::prelude::EventHandler;
/// struct Handler;
///
/// impl EventHandler for Handler {}
///
/// use serenity::Client;
/// use std::env;
/// use serenity::framework::StandardFramework;
/// use serenity::model::id::UserId;
///
/// let token = env::var("DISCORD_BOT_TOKEN").unwrap();
/// let mut client = Client::new(&token, Handler).unwrap();
///
/// client.with_framework(StandardFramework::new()
///     .configure(|c| c.on_mention(Some(UserId(5))).prefix("~")));
/// ```
///
/// [`Client`]: ../../client/struct.Client.html
/// [`StandardFramework`]: struct.StandardFramework.html
/// [default implementation]: #impl-Default
pub struct Configuration {
    #[doc(hidden)]
    pub allow_dm: bool,
    #[doc(hidden)]
    pub with_whitespace: WithWhiteSpace,
    #[doc(hidden)]
    pub by_space: bool,
    #[doc(hidden)]
    pub blocked_guilds: HashSet<GuildId>,
    #[doc(hidden)]
    pub blocked_users: HashSet<UserId>,
    #[doc(hidden)]
    pub allowed_channels: HashSet<ChannelId>,
    #[doc(hidden)]
    pub disabled_commands: HashSet<String>,
    #[doc(hidden)]
    pub dynamic_prefixes: Vec<Box<DynamicPrefixHook>>,
    #[doc(hidden)]
    pub ignore_bots: bool,
    #[doc(hidden)]
    pub ignore_webhooks: bool,
    #[doc(hidden)]
    pub on_mention: Option<String>,
    #[doc(hidden)]
    pub owners: HashSet<UserId>,
    #[doc(hidden)]
    pub prefixes: Vec<String>,
    #[doc(hidden)]
    pub no_dm_prefix: bool,
    #[doc(hidden)]
    pub delimiters: Vec<Delimiter>,
    #[doc(hidden)]
    pub case_insensitive: bool,
}

impl Configuration {
    /// If set to false, bot will ignore any private messages.
    ///
    /// **Note**: Defaults to `true`.
    pub fn allow_dm(&mut self, allow_dm: bool) -> &mut Self {
        self.allow_dm = allow_dm;

        self
    }

    /// Whether to allow whitespace being optional between a prefix/group-prefix/command and
    /// a command.
    ///
    /// **Note**: Defaults to `false` (for prefixes), `true` (commands), `true` (group prefixes).
    ///
    /// # Examples
    ///
    /// Setting `false` for prefixes will _only_ allow this scenario to occur:
    ///
    /// ```ignore
    /// !about
    ///
    /// // bot processes and executes the "about" command if it exists
    /// ```
    ///
    /// while setting it to `true` will _also_ allow this scenario to occur:
    ///
    /// ```ignore
    /// ! about
    ///
    /// // bot processes and executes the "about" command if it exists
    /// ```
    pub fn with_whitespace<I: Into<WithWhiteSpace>>(&mut self, with: I) -> &mut Self {
        self.with_whitespace = with.into();

        self
    }

    /// Whether the framework should split the message by a space first to parse the group or command.
    /// If set to false, it will only test part of the message by the *length* of the group's or command's names.
    ///
    /// **Note**: Defaults to `true`
    pub fn by_space(&mut self, b: bool) -> &mut Self {
        self.by_space = b;

        self
    }

       /// HashSet of channels Ids where commands will be working.
    ///
    /// **Note**: Defaults to an empty HashSet.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::model::id::ChannelId;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .allowed_channels(vec![ChannelId(7), ChannelId(77)].into_iter().collect())));
    /// ```
    pub fn allowed_channels(&mut self, channels: HashSet<ChannelId>) -> &mut Self {
        self.allowed_channels = channels;

        self
    }

    /// HashSet of guild Ids where commands will be ignored.
    ///
    /// **Note**: Defaults to an empty HashSet.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::model::id::GuildId;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .blocked_guilds(vec![GuildId(7), GuildId(77)].into_iter().collect())));
    /// ```
    pub fn blocked_guilds(&mut self, guilds: HashSet<GuildId>) -> &mut Self {
        self.blocked_guilds = guilds;

        self
    }

    /// HashSet of user Ids whose commands will be ignored.
    ///
    /// Guilds owned by user Ids will also be ignored.
    ///
    /// **Note**: Defaults to an empty HashSet.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::model::id::UserId;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .blocked_users(vec![UserId(7), UserId(77)].into_iter().collect())));
    /// ```
    pub fn blocked_users(&mut self, users: HashSet<UserId>) -> &mut Self {
        self.blocked_users = users;

        self
    }

    /// HashSet of command names that won't be run.
    ///
    /// **Note**: Defaults to an empty HashSet.
    ///
    /// # Examples
    ///
    /// Ignore a set of commands, assuming they exist:
    ///
    /// ```rust,no_run
    /// use serenity::framework::StandardFramework;
    /// use serenity::client::Context;
    /// use serenity::model::channel::Message;
    /// use serenity::framework::standard::{CommandResult, macros::{group, command}};
    ///
    /// #[command]
    /// fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    ///     msg.channel_id.say(&ctx.http, "Pong!")?;
    ///     Ok(())
    /// }
    ///
    /// #[group]
    /// #[commands(ping)]
    /// struct Peng;
    ///
    /// # fn main() {
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// let disabled = vec!["ping"].into_iter().map(|x| x.to_string()).collect();
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .group(&PENG_GROUP)
    ///     .configure(|c| c.disabled_commands(disabled)));
    /// # }
    /// ```
    pub fn disabled_commands(&mut self, commands: HashSet<String>) -> &mut Self {
        self.disabled_commands = commands;

        self
    }

    /// Sets the prefix to respond to dynamically based on conditions.
    ///
    /// Return `None` to not have a special prefix for the dispatch, and to
    /// instead use the inherited prefix.
    ///
    /// **Note**: Defaults to no dynamic prefix check.
    ///
    /// # Examples
    ///
    /// If the Id of the channel is divisible by 5, return a prefix of `"!"`,
    /// otherwise return a prefix of `"~"`.
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .configure(|c| c.dynamic_prefix(|_, msg| {
    ///         Some(if msg.channel_id.0 % 5 == 0 {
    ///             "!"
    ///         } else {
    ///             "~"
    ///         }.to_string())
    ///     })));
    /// ```
    pub fn dynamic_prefix<F>(&mut self, dynamic_prefix: F) -> &mut Self
    where
        F: Fn(&mut Context, &Message) -> Option<String> + Send + Sync + 'static,
    {
        self.dynamic_prefixes = vec![Box::new(dynamic_prefix)];

        self
    }

    #[inline]
    pub fn dynamic_prefixes<F, I: IntoIterator<Item = F>>(&mut self, iter: I) -> &mut Self
    where
        F: Fn(&mut Context, &Message) -> Option<String> + Send + Sync + 'static,
    {
        self.dynamic_prefixes = iter
            .into_iter()
            .map(|f| Box::new(f) as Box<DynamicPrefixHook>)
            .collect();

        self
    }

    /// Whether the bot should respond to other bots.
    ///
    /// For example, if this is set to false, then the bot will respond to any
    /// other bots including itself.
    ///
    /// **Note**: Defaults to `true`.
    pub fn ignore_bots(&mut self, ignore_bots: bool) -> &mut Self {
        self.ignore_bots = ignore_bots;

        self
    }

    /// If set to true, bot will ignore all commands called by webhooks.
    ///
    /// **Note**: Defaults to `true`.
    pub fn ignore_webhooks(&mut self, ignore_webhooks: bool) -> &mut Self {
        self.ignore_webhooks = ignore_webhooks;

        self
    }

    /// Whether or not to respond to commands initiated with `id_to_mention`.
    ///
    /// **Note**: that this can be used in conjunction with [`prefix`].
    ///
    /// **Note**: Defaults to ignore mentions.
    ///
    /// # Examples
    ///
    /// Setting this to an ID will allow the following types of mentions to be
    /// responded to:
    ///
    /// ```ignore
    /// <@245571012924538880> about
    /// <@!245571012924538880> about
    /// ```
    ///
    /// The former is a direct mention, while the latter is a nickname mention,
    /// which aids mobile devices in determining whether to display a user's
    /// nickname. It has no real meaning for your bot, and the library
    /// encourages you to ignore differentiating between the two.
    ///
    /// [`prefix`]: #method.prefix
    pub fn on_mention(&mut self, id_to_mention: Option<UserId>) -> &mut Self {
        self.on_mention = id_to_mention.map(|id| id.to_string());

        self
    }

    /// A `HashSet` of user Ids checks won't apply to.
    ///
    /// **Note**: Defaults to an empty HashSet.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::model::id::UserId;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .owners(vec![UserId(7), UserId(77)].into_iter().collect())));
    /// ```
    ///
    /// Create a HashSet beforehand:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::model::id::UserId;
    /// use std::collections::HashSet;
    /// use serenity::framework::StandardFramework;
    ///
    /// let mut set = HashSet::new();
    /// set.insert(UserId(7));
    /// set.insert(UserId(77));
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c.owners(set)));
    /// ```
    #[allow(clippy::implicit_hasher)]
    pub fn owners(&mut self, user_ids: HashSet<UserId>) -> &mut Self {
        self.owners = user_ids;

        self
    }

    /// Sets the prefix to respond to. A prefix can be a string slice of any
    /// non-zero length.
    ///
    /// **Note**: Defaults to an empty vector.
    ///
    /// # Examples
    ///
    /// Assign a basic prefix:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .prefix("!")));
    /// ```
    pub fn prefix(&mut self, prefix: &str) -> &mut Self {
        self.prefixes = vec![prefix.to_string()];

        self
    }

    /// Sets the prefixes to respond to. Each can be a string slice of any
    /// non-zero length.
    ///
    /// **Note**: Refer to [`prefix`] for the default value.
    ///
    /// # Examples
    ///
    /// Assign a set of prefixes the bot can respond to:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .prefixes(vec!["!", ">", "+"])));
    /// ```
    ///
    /// [`prefix`]: #method.prefix
    pub fn prefixes<T, It>(&mut self, prefixes: It) -> &mut Self
    where
        T: ToString,
        It: IntoIterator<Item = T>,
    {

        self.prefixes = prefixes.into_iter().map(|p| p.to_string()).collect();

        self
    }

    /// Sets whether command execution can done without a prefix. Works only in private channels.
    ///
    /// **Note**: Defaults to `false`.
    ///
    /// # Note
    ///
    /// The `cache` feature is required. If disabled this does absolutely nothing.
    pub fn no_dm_prefix(&mut self, b: bool) -> &mut Self {
        self.no_dm_prefix = b;

        self
    }

    /// Sets a single delimiter to be used when splitting the content after a command.
    ///
    /// **Note**: Defaults to a vector with a single element of `' '`.
    ///
    /// # Examples
    ///
    /// Have the args be separated by a comma and a space:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .delimiter(", ")));
    /// ```
    pub fn delimiter<I: Into<Delimiter>>(&mut self, delimiter: I) -> &mut Self {
        self.delimiters.clear();
        self.delimiters.push(delimiter.into());

        self
    }

    /// Sets multiple delimiters to be used when splitting the content after a command.
    /// Additionally cleans the default delimiter from the vector.
    ///
    /// **Note**: Refer to [`delimiter`] for the default value.
    ///
    /// # Examples
    ///
    /// Have the args be separated by a comma and a space; and a regular space:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .delimiters(vec![", ", " "])));
    /// ```
    ///
    /// [`delimiter`]: #method.delimiter
    pub fn delimiters<T, It>(&mut self, delimiters: It) -> &mut Self
    where
        T: Into<Delimiter>,
        It: IntoIterator<Item = T>,
    {
        self.delimiters.clear();
        self.delimiters
            .extend(delimiters.into_iter().map(|s| s.into()));

        self
    }

    /// Whether the framework shouldn't care about the user's input if it's:
    /// `~command`, `~Command`, or `~COMMAND`; `mayacommand`, `MayACommand`, `MAYACOMMAND`, et cetera.
    ///
    /// Setting this to `true` will result in *all* prefixes and command names to be case
    /// insensitive.
    ///
    /// **Note**: Defaults to `false`.
    pub fn case_insensitivity(&mut self, cs: bool) -> &mut Self {
        self.case_insensitive = cs;

        for prefix in &mut self.prefixes {
            *prefix = prefix.to_lowercase();
        }

        self
    }
}

impl Default for Configuration {
    /// Builds a default framework configuration, setting the following:
    ///
    /// - **allow_dm** to `true`
    /// - **with_whitespace** to `(false, true, true)`
    /// - **by_space** to `true`
    /// - **blocked_guilds** to an empty HashSet
    /// - **blocked_users** to an empty HashSet,
    /// - **allowed_channels** to an empty HashSet,
    /// - **case_insensitive** to `false`
    /// - **delimiters** to `vec![' ']`
    /// - **disabled_commands** to an empty HashSet
    /// - **dynamic_prefixes** to an empty vector
    /// - **ignore_bots** to `true`
    /// - **ignore_webhooks** to `true`
    /// - **no_dm_prefix** to `false`
    /// - **on_mention** to `false`
    /// - **owners** to an empty HashSet
    /// - **prefix** to an empty vector
    fn default() -> Configuration {
        Configuration {
            allow_dm: true,
            with_whitespace: WithWhiteSpace::default(),
            by_space: true,
            blocked_guilds: HashSet::default(),
            blocked_users: HashSet::default(),
            allowed_channels: HashSet::default(),
            case_insensitive: false,
            delimiters: vec![Delimiter::Single(' ')],
            disabled_commands: HashSet::default(),
            dynamic_prefixes: Vec::new(),
            ignore_bots: true,
            ignore_webhooks: true,
            no_dm_prefix: false,
            on_mention: None,
            owners: HashSet::default(),
            prefixes: vec![],
        }
    }
}
