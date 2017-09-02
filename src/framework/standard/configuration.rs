use std::collections::HashSet;
use std::default::Default;
use super::command::PrefixCheck;
use client::Context;
use http;
use model::{GuildId, Message, UserId};

/// The configuration to use for a [`Framework`] associated with a [`Client`]
/// instance.
///
/// This allows setting configurations like the depth to search for commands,
/// whether to treat mentions like a command prefix, etc.
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
/// use serenity::Client;
/// use std::env;
/// use serenity::framework::StandardFramework;
///
/// let mut client = Client::new(&env::var("DISCORD_BOT_TOKEN").unwrap(), Handler);
///
/// client.with_framework(StandardFramework::new()
///     .configure(|c| c.on_mention(true).prefix("~")));
/// ```
///
/// [`Client`]: ../../client/struct.Client.html
/// [`Framework`]: struct.Framework.html
pub struct Configuration {
    #[doc(hidden)]
    pub allow_dm: bool,
    #[doc(hidden)]
    pub allow_whitespace: bool,
    #[doc(hidden)]
    pub blocked_guilds: HashSet<GuildId>,
    #[doc(hidden)]
    pub blocked_users: HashSet<UserId>,
    #[doc(hidden)]
    pub depth: usize,
    #[doc(hidden)]
    pub disabled_commands: HashSet<String>,
    #[doc(hidden)]
    pub dynamic_prefix: Option<Box<PrefixCheck>>,
    #[doc(hidden)]
    pub ignore_bots: bool,
    #[doc(hidden)]
    pub ignore_webhooks: bool,
    #[doc(hidden)]
    pub on_mention: Option<Vec<String>>,
    #[doc(hidden)]
    pub owners: HashSet<UserId>,
    #[doc(hidden)]
    pub prefixes: Vec<String>,
    #[doc(hidden)]
    pub delimiters: Vec<String>,
    #[doc(hidden)]
    pub case_insensitive: bool,
}

impl Configuration {
    /// If set to false, bot will ignore any private messages.
    pub fn allow_dm(mut self, allow_dm: bool) -> Self {
        self.allow_dm = allow_dm;

        self
    }

    /// Whether to allow whitespace being optional between a mention/prefix and
    /// a command.
    ///
    /// **Note**: Defaults to `false`.
    ///
    /// # Examples
    ///
    /// Setting this to `false` will _only_ allow this scenario to occur:
    ///
    /// ```ignore
    /// <@245571012924538880> about
    /// !about
    ///
    /// // bot processes and executes the "about" command if it exists
    /// ```
    ///
    /// while setting this to `true` will _also_ allow this scenario to occur:
    ///
    /// ```ignore
    /// <@245571012924538880>about
    /// ! about
    ///
    /// // bot processes and executes the "about" command if it exists
    /// ```
    pub fn allow_whitespace(mut self, allow_whitespace: bool) -> Self {
        self.allow_whitespace = allow_whitespace;

        self
    }

    /// HashSet of guild Ids where commands will be ignored.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// use serenity::model::GuildId;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .blocked_guilds(vec![GuildId(7), GuildId(77)].into_iter().collect())));
    /// ```
    pub fn blocked_guilds(mut self, guilds: HashSet<GuildId>) -> Self {
        self.blocked_guilds = guilds;

        self
    }

    /// HashSet of user Ids whose commands will be ignored.
    /// Guilds owned by user Ids will also be ignored.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// use serenity::model::UserId;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .blocked_users(vec![UserId(7), UserId(77)].into_iter().collect())));
    /// ```
    pub fn blocked_users(mut self, users: HashSet<UserId>) -> Self {
        self.blocked_users = users;

        self
    }

    /// The default depth of the message to check for commands. Defaults to 5.
    /// This determines how "far" into a message to check for a valid command.
    ///
    /// # Examples
    ///
    /// If you set a depth of `1`, and make a command of `"music play"`, but
    /// not a `"music"` command, then the former command will never be
    /// triggered, as its "depth" is `2`.
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = depth as usize;

        self
    }

    /// HashSet of command names that won't be run.
    ///
    /// # Examples
    ///
    /// Ignore a set of commands, assuming they exist:
    ///
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// use serenity::framework::StandardFramework;
    ///
    /// let disabled = vec!["ping"].into_iter().map(|x| x.to_owned()).collect();
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .command("ping", |c| c.exec_str("pong!"))
    ///     .configure(|c| c.disabled_commands(disabled)));
    /// ```
    pub fn disabled_commands(mut self, commands: HashSet<String>) -> Self {
        self.disabled_commands = commands;

        self
    }

    /// Sets the prefix to respond to dynamically based on conditions.
    ///
    /// Return `None` to not have a special prefix for the dispatch, and to
    /// instead use the inherited prefix.
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
    /// # let mut client = Client::new("token", Handler);
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .command("ping", |c| c.exec_str("Pong!"))
    ///     .configure(|c| c.dynamic_prefix(|_, msg| {
    ///         Some(if msg.channel_id.0 % 5 == 0 {
    ///             "!"
    ///         } else {
    ///             "~"
    ///         }.to_owned())
    ///     })));
    /// ```
    pub fn dynamic_prefix<F>(mut self, dynamic_prefix: F) -> Self
        where F: Fn(&mut Context, &Message) -> Option<String> + Send + Sync + 'static {
        self.dynamic_prefix = Some(Box::new(dynamic_prefix));

        self
    }

    /// Whether the bot should respond to other bots.
    ///
    /// For example, if this is set to false, then the bot will respond to any
    /// other bots including itself.
    pub fn ignore_bots(mut self, ignore_bots: bool) -> Self {
        self.ignore_bots = ignore_bots;

        self
    }

    /// If set to true, bot will ignore all commands called by webhooks.
    /// True by default.
    pub fn ignore_webhooks(mut self, ignore_webhooks: bool) -> Self {
        self.ignore_webhooks = ignore_webhooks;

        self
    }

    /// Whether or not to respond to commands initiated with a mention. Note
    /// that this can be used in conjunction with [`prefix`].
    ///
    /// By default this is set to `false`.
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
    pub fn on_mention(mut self, on_mention: bool) -> Self {
        if !on_mention {
            return self;
        }

        if let Ok(current_user) = http::get_current_user() {
            self.on_mention = Some(vec![
                format!("<@{}>", current_user.id), // Regular mention
                format!("<@!{}>", current_user.id) /* Nickname mention */,
            ]);
        }

        self
    }

    /// A `HashSet` of user Ids checks won't apply to.
    ///
    /// # Examples
    ///
    /// Create a HashSet in-place:
    ///
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// use serenity::model::UserId;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .owners(vec![UserId(7), UserId(77)].into_iter().collect())));
    /// ```
    ///
    /// Create a HashSet beforehand:
    ///
    /// ```rust
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler);
    /// use serenity::model::UserId;
    /// use std::collections::HashSet;
    /// use serenity::framework::StandardFramework;
    ///
    /// let mut set = HashSet::new();
    /// set.insert(UserId(7));
    /// set.insert(UserId(77));
    ///
    /// client.with_framework(StandardFramework::new().configure(|c| c.owners(set)));
    /// ```
    pub fn owners(mut self, user_ids: HashSet<UserId>) -> Self {
        self.owners = user_ids;

        self
    }

    /// Sets the prefix to respond to. A prefix can be a string slice of any
    /// non-zero length.
    ///
    /// # Examples
    ///
    /// Assign a basic prefix:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .prefix("!")));
    /// ```
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefixes = vec![prefix.to_owned()];

        self
    }

    /// Sets the prefixes to respond to. Each can be a string slice of any
    /// non-zero length.
    ///
    /// # Examples
    ///
    /// Assign a set of prefixes the bot can respond to:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .prefixes(vec!["!", ">", "+"])));
    /// ```
    pub fn prefixes(mut self, prefixes: Vec<&str>) -> Self {
        self.prefixes = prefixes.iter().map(|x| x.to_string()).collect();

        self
    }

    /// Sets a delimiter to be used when splitting the content after a command.
    ///
    /// # Examples
    ///
    /// Have the args be seperated by a comma and a space:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .delimiter(", ")));
    /// ```
    pub fn delimiter(mut self, delimiter: &str) -> Self {
        self.delimiters.push(delimiter.to_string());

        self
    }

    /// Sets multiple delimiters to be used when splitting the content after a command.
    /// Additionally cleans the default delimiter from the vector.
    ///
    /// # Examples
    ///
    /// Have the args be seperated by a comma and a space; and a regular space:
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
    /// client.with_framework(StandardFramework::new().configure(|c| c
    ///     .delimiters(vec![", ", " "])));
    /// ```
    pub fn delimiters(mut self, delimiters: Vec<&str>) -> Self {
        self.delimiters.clear();
        self.delimiters.extend(
            delimiters.into_iter().map(|s| s.to_string()),
        );

        self
    }

    /// Whether the framework shouldn't care about the user's input if it's: `~command`,
    /// `~Command`, `~COMMAND`.
    /// Setting this to `true` will result in *all* command names to be case insensitive.
    pub fn case_insensitivity(mut self, cs: bool) -> Self {
        self.case_insensitive = cs;

        self
    }
}

impl Default for Configuration {
    /// Builds a default framework configuration, setting the following:
    ///
    /// - **allow_whitespace** to `false`
    /// - **depth** to `5`
    /// - **on_mention** to `false` (basically)
    /// - **prefix** to `None`
    /// - **delimiters** to vec![" "]
    /// - **case_insensitive** to `false`
    fn default() -> Configuration {
        Configuration {
            depth: 5,
            on_mention: None,
            dynamic_prefix: None,
            allow_whitespace: false,
            prefixes: vec![],
            ignore_bots: true,
            owners: HashSet::default(),
            blocked_users: HashSet::default(),
            blocked_guilds: HashSet::default(),
            disabled_commands: HashSet::default(),
            allow_dm: true,
            ignore_webhooks: true,
            case_insensitive: false,
            delimiters: vec![" ".to_string()],
        }
    }
}
