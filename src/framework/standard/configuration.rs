use std::collections::HashSet;

use derivative::Derivative;
use futures::future::BoxFuture;

use super::Delimiter;
use crate::client::Context;
use crate::model::channel::Message;
use crate::model::id::{ChannelId, GuildId, UserId};

type DynamicPrefixHook<D> =
    for<'fut> fn(&'fut Context<D>, &'fut Message) -> BoxFuture<'fut, Option<String>>;

/// A configuration struct for deciding whether the framework should allow optional whitespace
/// between prefixes, group prefixes and command names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// The configuration to use for a [`StandardFramework`] associated with a [`Client`] instance.
///
/// This allows setting configurations like the depth to search for commands, whether to treat
/// mentions like a command prefix, etc.
///
/// To see the default values, refer to the [default implementation].
///
/// # Examples
///
/// Responding to mentions and setting a command prefix of `"~"`:
///
/// ```rust,no_run
/// # use serenity::prelude::*;
/// struct Handler;
///
/// impl EventHandler<()> for Handler {}
///
/// use serenity::framework::standard::{Configuration, StandardFramework};
/// use serenity::model::id::UserId;
/// use serenity::Client;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let token = std::env::var("DISCORD_BOT_TOKEN")?;
///
/// let framework = StandardFramework::new();
/// framework.configure(Configuration::new().on_mention(Some(UserId::new(5))).prefix("~"));
///
/// let mut client = Client::builder(&token, GatewayIntents::default(), ())
///     .event_handler(Handler)
///     .framework(framework)
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// [`Client`]: crate::Client
/// [`StandardFramework`]: super::StandardFramework
/// [default implementation]: Self::default
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct Configuration<D: Send + Sync + 'static> {
    pub(crate) allow_dm: bool,
    pub(crate) with_whitespace: WithWhiteSpace,
    pub(crate) by_space: bool,
    pub(crate) blocked_guilds: HashSet<GuildId>,
    pub(crate) blocked_users: HashSet<UserId>,
    pub(crate) allowed_channels: HashSet<ChannelId>,
    pub(crate) disabled_commands: HashSet<String>,
    pub(crate) dynamic_prefixes: Vec<DynamicPrefixHook<D>>,
    pub(crate) ignore_bots: bool,
    pub(crate) ignore_webhooks: bool,
    pub(crate) on_mention: Option<String>,
    pub(crate) owners: HashSet<UserId>,
    pub(crate) prefixes: Vec<String>,
    pub(crate) no_dm_prefix: bool,
    pub(crate) delimiters: Vec<Delimiter>,
    pub(crate) case_insensitive: bool,
}

impl<D: Send + Sync + 'static> Configuration<D> {
    /// Alias for Configuration::default
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// If set to false, bot will ignore any private messages.
    ///
    /// **Note**: Defaults to `true`.
    #[must_use]
    pub fn allow_dm(mut self, allow_dm: bool) -> Self {
        self.allow_dm = allow_dm;
        self
    }

    /// Whether to allow whitespace being optional between a prefix/group-prefix/command and a
    /// command.
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
    #[must_use]
    pub fn with_whitespace(mut self, with: impl Into<WithWhiteSpace>) -> Self {
        self.with_whitespace = with.into();
        self
    }

    /// Whether the framework should split the message by a space first to parse the group or
    /// command. If set to false, it will only test part of the message by the *length* of the
    /// group's or command's names.
    ///
    /// **Note**: Defaults to `true`
    #[must_use]
    pub fn by_space(mut self, b: bool) -> Self {
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
    /// use serenity::framework::standard::{Configuration, StandardFramework};
    /// use serenity::model::id::ChannelId;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new()
    ///     .allowed_channels(vec![ChannelId::new(7), ChannelId::new(77)].into_iter().collect());
    /// ```
    #[must_use]
    pub fn allowed_channels(mut self, channels: HashSet<ChannelId>) -> Self {
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
    /// use serenity::framework::standard::{Configuration, StandardFramework};
    /// use serenity::model::id::GuildId;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new()
    ///     .blocked_guilds(vec![GuildId::new(7), GuildId::new(77)].into_iter().collect());
    /// ```
    #[must_use]
    pub fn blocked_guilds(mut self, guilds: HashSet<GuildId>) -> Self {
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
    /// use serenity::framework::standard::{Configuration, StandardFramework};
    /// use serenity::model::id::UserId;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().blocked_users(vec![UserId::new(7), UserId::new(77)].into_iter().collect());
    /// ```
    #[must_use]
    pub fn blocked_users(mut self, users: HashSet<UserId>) -> Self {
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
    /// # type Data = ();
    /// # type Context = serenity::client::Context<Data>;
    /// use serenity::framework::standard::macros::{command, group};
    /// use serenity::framework::standard::{CommandResult, Configuration};
    /// use serenity::framework::StandardFramework;
    /// use serenity::model::channel::Message;
    ///
    /// #[command]
    /// async fn ping<Data>(ctx: &Context, msg: &Message) -> CommandResult {
    ///     msg.channel_id.say(&ctx.http, "Pong!").await?;
    ///     Ok(())
    /// }
    ///
    /// #[group]
    /// #[commands(ping)]
    /// struct Peng<Data>;
    ///
    /// let disabled = vec!["ping"].into_iter().map(|x| x.to_string()).collect();
    ///
    /// let framework = StandardFramework::new().group(&PENG_GROUP);
    /// framework.configure(Configuration::new().disabled_commands(disabled));
    /// ```
    #[inline]
    #[must_use]
    pub fn disabled_commands(mut self, commands: HashSet<String>) -> Self {
        self.disabled_commands = commands;
        self
    }

    /// Sets the prefix to respond to dynamically, in addition to the one configured with
    /// [`Self::prefix`] or [`Self::prefixes`]. This is useful if you want to have user
    /// configurable per-guild or per-user prefixes, such as by fetching a guild's prefix from a
    /// database accessible via [`Context::data`].
    ///
    /// Return [`None`] to not have a special prefix for the dispatch and to only use the
    /// configured prefix from [`Self::prefix`] or [`Self::prefixes`].
    ///
    /// This method can be called many times to add more dynamic prefix hooks.
    ///
    /// **Note**: Defaults to no dynamic prefix check.
    ///
    /// **Note**: If using dynamic_prefix *without* [`Self::prefix`] or [`Self::prefixes`], there
    /// will still be the default framework prefix of `"~"`. You can disable the default prefix by
    /// setting the prefix to an empty string `""` with [`Self::prefix`].
    ///
    /// # Examples
    ///
    /// If the Id of the channel is divisible by 5, use the prefix `"!"`, otherwise use `"*"`. The
    /// default framework prefix `"~"` will always be valid in addition to the one returned by
    /// dynamic_prefix.
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// use serenity::framework::standard::Configuration;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().dynamic_prefix(|_, msg| {
    ///     Box::pin(
    ///         async move { Some(if msg.channel_id.get() % 5 == 0 { "!" } else { "*" }.to_string()) },
    ///     )
    /// });
    /// ```
    ///
    /// This will only use the prefix `"!"` or `"*"` depending on channel ID,
    /// with the default prefix `"~"` disabled.
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// use serenity::framework::standard::Configuration;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new()
    ///     .dynamic_prefix(|_, msg| {
    ///         Box::pin(async move {
    ///             Some(if msg.channel_id.get() % 5 == 0 { "!" } else { "*" }.to_string())
    ///         })
    ///     })
    ///     .prefix(""); // This disables the default prefix "~"
    /// ```
    ///
    /// [`Context::data`]: crate::client::Context::data
    #[inline]
    #[must_use]
    pub fn dynamic_prefix(mut self, dynamic_prefix: DynamicPrefixHook<D>) -> Self {
        self.dynamic_prefixes.push(dynamic_prefix);
        self
    }

    /// Whether the bot should respond to other bots.
    ///
    /// For example, if this is set to false, then the bot will respond to any other bots including
    /// itself.
    ///
    /// **Note**: Defaults to `true`.
    #[must_use]
    pub fn ignore_bots(mut self, ignore_bots: bool) -> Self {
        self.ignore_bots = ignore_bots;
        self
    }

    /// If set to true, bot will ignore all commands called by webhooks.
    ///
    /// **Note**: Defaults to `true`.
    #[must_use]
    pub fn ignore_webhooks(mut self, ignore_webhooks: bool) -> Self {
        self.ignore_webhooks = ignore_webhooks;
        self
    }

    /// Whether or not to respond to commands initiated with `id_to_mention`.
    ///
    /// **Note**: that this can be used in conjunction with [`Self::prefix`].
    ///
    /// **Note**: Defaults to ignore mentions.
    ///
    /// # Examples
    ///
    /// Setting this to an ID will allow the following types of mentions to be responded to:
    ///
    /// ```ignore
    /// <@245571012924538880> about
    /// <@!245571012924538880> about
    /// ```
    ///
    /// The former is a direct mention, while the latter is a nickname mention, which aids mobile
    /// devices in determining whether to display a user's nickname. It has no real meaning for
    /// your bot, and the library encourages you to ignore differentiating between the two.
    #[must_use]
    pub fn on_mention(mut self, id_to_mention: Option<UserId>) -> Self {
        self.on_mention = id_to_mention.map(|id| id.to_string());
        self
    }

    /// A [`HashSet`] of user Ids checks won't apply to.
    ///
    /// **Note**: Defaults to an empty HashSet.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust,no_run
    /// use serenity::framework::standard::Configuration;
    /// use serenity::model::id::UserId;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().owners(vec![UserId::new(7), UserId::new(77)].into_iter().collect());
    /// ```
    ///
    /// Create a HashSet beforehand:
    ///
    /// ```rust,no_run
    /// use std::collections::HashSet;
    ///
    /// use serenity::framework::standard::Configuration;
    /// use serenity::model::id::UserId;
    ///
    /// let mut set = HashSet::new();
    /// set.insert(UserId::new(7));
    /// set.insert(UserId::new(77));
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().owners(set);
    /// ```
    #[must_use]
    pub fn owners(mut self, user_ids: HashSet<UserId>) -> Self {
        self.owners = user_ids;
        self
    }

    /// Sets the prefix to respond to. A prefix can be a string slice of any non-zero length.
    ///
    /// **Note**: Defaults to "~".
    ///
    /// **Note**: Passing empty string `""` will set no prefix.
    ///
    /// **Note**: This prefix will always be usable, even if there is a [`Self::dynamic_prefix`]
    /// configured.
    ///
    /// # Examples
    ///
    /// Assign a basic prefix:
    ///
    /// ```rust,no_run
    /// use serenity::framework::standard::Configuration;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().prefix("!");
    /// ```
    #[must_use]
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        let p = prefix.into();
        self.prefixes = if p.is_empty() { vec![] } else { vec![p] };
        self
    }

    /// Sets the prefixes to respond to. Each can be a string slice of any non-zero length.
    ///
    /// **Note**: Refer to [`Self::prefix`] for the default value.
    ///
    /// **Note**: These prefixes will always be usable, even if there is a [`Self::dynamic_prefix`]
    /// configured.
    ///
    /// # Examples
    ///
    /// Assign a set of prefixes the bot can respond to:
    ///
    /// ```rust,no_run
    /// use serenity::framework::standard::Configuration;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().prefixes(vec!["!", ">", "+"]);
    /// ```
    #[inline]
    #[must_use]
    pub fn prefixes(mut self, prefixes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.prefixes = prefixes.into_iter().map(Into::into).filter(|p| !p.is_empty()).collect();
        self
    }

    /// Sets whether command execution can be done without a prefix. Works only in private channels.
    ///
    /// **Note**: Defaults to `false`.
    ///
    /// # Note
    ///
    /// The `cache` feature is required. If disabled this does absolutely nothing.
    #[inline]
    #[must_use]
    pub fn no_dm_prefix(mut self, b: bool) -> Self {
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
    /// use serenity::framework::standard::Configuration;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().delimiter(", ");
    /// ```
    #[must_use]
    pub fn delimiter(mut self, delimiter: impl Into<Delimiter>) -> Self {
        self.delimiters.clear();
        self.delimiters.push(delimiter.into());

        self
    }

    /// Sets multiple delimiters to be used when splitting the content after a command.
    /// Additionally cleans the default delimiter from the vector.
    ///
    /// **Note**: Refer to [`Self::delimiter`] for the default value.
    ///
    /// # Examples
    ///
    /// Have the args be separated by a comma and a space; and a regular space:
    ///
    /// ```rust,no_run
    /// use serenity::framework::standard::Configuration;
    ///
    /// # let config: Configuration<()> =
    /// Configuration::new().delimiters(vec![", ", " "]);
    /// ```
    #[must_use]
    pub fn delimiters(
        mut self,
        delimiters: impl IntoIterator<Item = impl Into<Delimiter>>,
    ) -> Self {
        self.delimiters.clear();
        self.delimiters.extend(delimiters.into_iter().map(Into::into));

        self
    }

    /// Whether the framework shouldn't care about the user's input if it's: `~command`,
    /// `~Command`, or `~COMMAND`; `mayacommand`, `MayACommand`, `MAYACOMMAND`, et cetera.
    ///
    /// Setting this to `true` will result in *all* prefixes and command names to be case
    /// insensitive.
    ///
    /// **Note**: Defaults to `false`.
    #[must_use]
    pub fn case_insensitivity(mut self, cs: bool) -> Self {
        self.case_insensitive = cs;

        for prefix in &mut self.prefixes {
            *prefix = prefix.to_lowercase();
        }

        self
    }
}

impl<D: Send + Sync + 'static> Default for Configuration<D> {
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
    /// - **prefix** to "~"
    fn default() -> Self {
        Self {
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
            prefixes: vec![String::from("~")],
        }
    }
}
