use super::Delimiter;
use crate::client::Context;
use crate::http::Http;
use crate::model::{channel::Message, id::UserId};
use std::collections::HashSet;

type DynamicPrefixHook = Fn(&mut Context, &Message) -> Option<String> + Send + Sync + 'static;

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
///
/// let token = env::var("DISCORD_BOT_TOKEN").unwrap();
/// let mut client = Client::new(&token, Handler).unwrap();
///
/// client.with_framework(StandardFramework::new()
///     .configure(|c| c.on_mention(true).prefix("~")));
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
    /// group!({
    ///     name: "peng",
    ///     options: {},
    ///     commands: [ping]
    /// });
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

    /// Whether or not to respond to commands initiated with a mention. Note
    /// that this can be used in conjunction with [`prefix`].
    ///
    /// **Note**: Defaults to `false`.
    ///
    /// # Examples
    ///
    /// Setting this to `true` will allow the following types of mentions to be
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
    pub fn on_mention(&mut self, on_mention: bool) -> &mut Self {
        if !on_mention {
            return self;
        }

        let http = Http::new(
            reqwest::Client::builder().build().expect("Could not construct Reqwest-Client."),
            "",
        );

        if let Ok(current_user) = http.get_current_user() {
            self.on_mention = Some(current_user.id.to_string());
        }

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
        self.prefixes = prefixes.into_iter().map(|x| x.to_string()).collect();

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
    /// `~command`, `~Command`, or `~COMMAND`.
    ///
    /// Setting this to `true` will result in *all* command names to be case
    /// insensitive.
    ///
    /// **Note**: Defaults to `false`.
    pub fn case_insensitivity(&mut self, cs: bool) -> &mut Self {
        self.case_insensitive = cs;

        self
    }
}

impl Default for Configuration {
    /// Builds a default framework configuration, setting the following:
    ///
    /// - **allow_dm** to `true`
    /// - **with_whitespace** to `(false, true, true)`
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
