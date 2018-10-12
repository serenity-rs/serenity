use client::Context;
use http;
use model::{
    channel::Message,
    id::{ChannelId, GuildId, UserId}
};
use std::{
    collections::HashSet,
    default::Default,
    sync::Arc,
};
use super::command::{Command, InternalCommand, PrefixCheck};

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
    #[doc(hidden)] pub allow_dm: bool,
    #[doc(hidden)] pub allow_whitespace: bool,
    #[doc(hidden)] pub blocked_guilds: HashSet<GuildId>,
    #[doc(hidden)] pub blocked_users: HashSet<UserId>,
    #[doc(hidden)] pub allowed_channels: HashSet<ChannelId>,
    #[doc(hidden)] pub depth: usize,
    #[doc(hidden)] pub disabled_commands: HashSet<String>,
    #[doc(hidden)] pub dynamic_prefix: Option<Box<PrefixCheck>>,
    #[doc(hidden)] pub ignore_bots: bool,
    #[doc(hidden)] pub ignore_webhooks: bool,
    #[doc(hidden)] pub on_mention: Option<Vec<String>>,
    #[doc(hidden)] pub owners: HashSet<UserId>,
    #[doc(hidden)] pub prefixes: Vec<String>,
    #[doc(hidden)] pub no_dm_prefix: bool,
    #[doc(hidden)] pub delimiters: Vec<String>,
    #[doc(hidden)] pub case_insensitive: bool,
    #[doc(hidden)] pub prefix_only_cmd: Option<InternalCommand>,
}

impl Configuration {
    /// If set to false, bot will ignore any private messages.
    ///
    /// **Note**: Defaults to `true`.
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
    pub fn blocked_users(mut self, users: HashSet<UserId>) -> Self {
        self.blocked_users = users;

        self
    }

    /// The default depth of the message to check for commands.
    ///
    /// This determines how "far" into a message to check for a valid command.
    ///
    /// **Note**: Defaults to 5.
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
    /// **Note**: Defaults to an empty HashSet.
    ///
    /// # Examples
    ///
    /// Ignore a set of commands, assuming they exist:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # struct Handler;
    /// #
    /// # impl EventHandler for Handler {}
    /// # let mut client = Client::new("token", Handler).unwrap();
    /// use serenity::framework::StandardFramework;
    ///
    /// let disabled = vec!["ping"].into_iter().map(|x| x.to_string()).collect();
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .on("ping", |_, msg, _| {
    ///         msg.channel_id.say("Pong!")?;
    ///
    ///         Ok(())
    ///     })
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
    ///     .on("ping", |_, msg, _| {
    ///         msg.channel_id.say("Pong!")?;
    ///
    ///         Ok(())
    ///      })
    ///     .configure(|c| c.dynamic_prefix(|_, msg| {
    ///         Some(if msg.channel_id.0 % 5 == 0 {
    ///             "!"
    ///         } else {
    ///             "~"
    ///         }.to_string())
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
    ///
    /// **Note**: Defaults to `true`.
    pub fn ignore_bots(mut self, ignore_bots: bool) -> Self {
        self.ignore_bots = ignore_bots;

        self
    }

    /// If set to true, bot will ignore all commands called by webhooks.
    ///
    /// **Note**: Defaults to `true`.
    pub fn ignore_webhooks(mut self, ignore_webhooks: bool) -> Self {
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
    pub fn on_mention(mut self, on_mention: bool) -> Self {
        if !on_mention {
            return self;
        }

        if let Ok(current_user) = http::get_current_user() {
            self.on_mention = Some(vec![
                format!("<@{}>", current_user.id),  // Regular mention
                format!("<@!{}>", current_user.id), // Nickname mention
            ]);
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
    pub fn owners(mut self, user_ids: HashSet<UserId>) -> Self {
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
    pub fn prefix(mut self, prefix: &str) -> Self {
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
    pub fn prefixes<T: ToString, It: IntoIterator<Item=T>>(mut self, prefixes: It) -> Self {
        self.prefixes = prefixes.into_iter().map(|x| x.to_string()).collect();

        self
    }

    /// Sets whether command execution can done without a prefix. Works only in private channels.
    ///
    /// **Note**: Defaults to `false`.
    ///
    /// # Note
    ///
    /// Needs the `cache` feature to be enabled. Otherwise this does nothing.
    pub fn no_dm_prefix(mut self, b: bool) -> Self {
        self.no_dm_prefix = b;

        self
    }

    /// Sets a delimiter to be used when splitting the content after a command.
    ///
    /// **Note**: Defaults to a vector with a single element of `" "`.
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
    pub fn delimiter(mut self, delimiter: &str) -> Self {
        self.delimiters.push(delimiter.to_string());

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
    pub fn delimiters<T: ToString, It: IntoIterator<Item=T>>(mut self, delimiters: It) -> Self {
        self.delimiters.clear();
        self.delimiters
            .extend(delimiters.into_iter().map(|s| s.to_string()));

        self
    }

    /// Whether the framework shouldn't care about the user's input if it's:
    /// `~command`, `~Command`, or `~COMMAND`.
    ///
    /// Setting this to `true` will result in *all* command names to be case
    /// insensitive.
    ///
    /// **Note**: Defaults to `false`.
    pub fn case_insensitivity(mut self, cs: bool) -> Self {
        self.case_insensitive = cs;

        self
    }

    /// Sets a command to dispatch if user's input is a prefix only.
    ///
    /// **Note**: Defaults to no command and ignores prefix only.
    pub fn prefix_only_cmd<C: Command + 'static>(mut self, c: C) -> Self {
        self.prefix_only_cmd = Some(Arc::new(c));

        self
    }
}

impl Default for Configuration {
    /// Builds a default framework configuration, setting the following:
    ///
    /// - **allow_dm** to `true`
    /// - **allow_whitespace** to `false`
    /// - **allowed_channels** to an empty HashSet
    /// - **blocked_guilds** to an empty HashSet
    /// - **blocked_users** to an empty HashSet
    /// - **case_insensitive** to `false`
    /// - **delimiters** to `vec![" "]`
    /// - **depth** to `5`
    /// - **disabled_commands** to an empty HashSet
    /// - **dynamic_prefix** to no dynamic prefix check
    /// - **ignore_bots** to `true`
    /// - **ignore_webhooks** to `true`
    /// - **no_dm_prefix** to `false`
    /// - **on_mention** to `false` (basically)
    /// - **owners** to an empty HashSet
    /// - **prefix** to an empty vector
    fn default() -> Configuration {
        Configuration {
            allow_dm: true,
            allow_whitespace: false,
            allowed_channels: HashSet::default(),
            blocked_guilds: HashSet::default(),
            blocked_users: HashSet::default(),
            case_insensitive: false,
            delimiters: vec![" ".to_string()],
            depth: 5,
            disabled_commands: HashSet::default(),
            dynamic_prefix: None,
            ignore_bots: true,
            ignore_webhooks: true,
            no_dm_prefix: false,
            on_mention: None,
            owners: HashSet::default(),
            prefixes: vec![],
            prefix_only_cmd: None,
        }
    }
}
