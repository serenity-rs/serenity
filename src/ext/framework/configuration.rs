use std::collections::HashSet;
use std::default::Default;
use super::command::PrefixCheck;
use ::client::{Context, rest};
use ::model::{GuildId, UserId};

/// Account type used for configuration.
pub enum AccountType {
    /// Connected client will only listen to itself.
    Selfbot,
    /// Connected client will ignore all bot accounts.
    Bot,
    /// Connected client will listen to everyone.
    Any,
    #[doc(hidden)]
    Automatic
}

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
/// use serenity::Client;
/// use std::env;
///
/// let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN").unwrap());
///
/// client.with_framework(|f| f
///     .configure(|c| c.on_mention(true).prefix("~")));
/// ```
///
/// [`Client`]: ../../client/struct.Client.html
/// [`Framework`]: struct.Framework.html
pub struct Configuration {
    #[doc(hidden)]
    pub depth: usize,
    #[doc(hidden)]
    pub on_mention: Option<Vec<String>>,
    #[doc(hidden)]
    pub allow_whitespace: bool,
    #[doc(hidden)]
    pub prefixes: Vec<String>,
    #[doc(hidden)]
    pub dynamic_prefix: Option<Box<PrefixCheck>>,
    #[doc(hidden)]
    pub rate_limit_message: Option<String>,
    #[doc(hidden)]
    pub invalid_permission_message: Option<String>,
    #[doc(hidden)]
    pub invalid_check_message: Option<String>,
    #[doc(hidden)]
    pub no_dm_message: Option<String>,
    #[doc(hidden)]
    pub no_guild_message: Option<String>,
    #[doc(hidden)]
    pub blocked_user_message: Option<String>,
    #[doc(hidden)]
    pub blocked_guild_message: Option<String>,
    #[doc(hidden)]
    pub too_many_args_message: Option<String>,
    #[doc(hidden)]
    pub not_enough_args_message: Option<String>,
    #[doc(hidden)]
    pub command_disabled_message: Option<String>,
    #[doc(hidden)]
    pub account_type: AccountType,
    #[doc(hidden)]
    pub blocked_users: HashSet<UserId>,
    #[doc(hidden)]
    pub blocked_guilds: HashSet<GuildId>,
    #[doc(hidden)]
    pub owners: HashSet<UserId>,
    #[doc(hidden)]
    pub disabled_commands: HashSet<String>,
    #[doc(hidden)]
    pub allow_dm: bool,
}

impl Configuration {
    /// Allows you to change what accounts to ignore.
    pub fn account_type(mut self, account_type: AccountType) -> Self {
        self.account_type = account_type;

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

    /// HashSet of user Ids whose commands will be ignored.
    /// Guilds owned by user Ids will also be ignored.
    pub fn blocked_users(mut self, users: HashSet<UserId>) -> Self {
        self.blocked_users = users;

        self
    }

    /// HashSet of guild Ids where commands will be ignored.
    pub fn blocked_guilds(mut self, guilds: HashSet<GuildId>) -> Self {
        self.blocked_guilds = guilds;

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
    pub fn disabled_commands(mut self, commands: HashSet<String>) -> Self {
        self.disabled_commands = commands;

        self
    }

    /// Sets the prefix to respond to. This can either be a single- or
    /// multi-char string.
    pub fn dynamic_prefix<F>(mut self, dynamic_prefix: F) -> Self
        where F: Fn(&Context) -> Option<String> + Send + Sync + 'static {
        self.dynamic_prefix = Some(Box::new(dynamic_prefix));

        self
    }

    /// Message that's sent when one of a command's checks doesn't succeed.
    pub fn invalid_check_message(mut self, content: &str) -> Self {
        self.invalid_check_message = Some(content.to_owned());

        self
    }

    /// Message that's sent if a command is disabled.
    ///
    /// %command% will be replaced with command name.
    pub fn command_disabled_message(mut self, content: &str) -> Self {
        self.command_disabled_message = Some(content.to_owned());

        self
    }

    /// Message that's sent if a blocked user attempts to use a command.
    pub fn blocked_user_message(mut self, content: &str) -> Self {
        self.blocked_user_message = Some(content.to_owned());

        self
    }

    /// Message that's sent if a command issued within a guild owned by a
    /// blocked user.
    pub fn blocked_guild_message(mut self, content: &str) -> Self {
        self.blocked_guild_message = Some(content.to_owned());

        self
    }

    /// Message that's sent when a user with wrong permissions calls a command.
    pub fn invalid_permission_message(mut self, content: &str) -> Self {
        self.invalid_permission_message = Some(content.to_owned());

        self
    }

    /// Message that's sent when a command is on cooldown.
    /// See framework documentation to see where is this utilized.
    ///
    /// %time% will be replaced with waiting time in seconds.
    pub fn rate_limit_message(mut self, content: &str) -> Self {
        self.rate_limit_message = Some(content.to_owned());

        self
    }

    /// Message that's sent when a command isn't available in DM.
    pub fn no_dm_message(mut self, content: &str) -> Self {
        self.no_dm_message = Some(content.to_owned());

        self
    }

    /// Message that's sent when a command isn't available in guilds.
    pub fn no_guild_message(mut self, content: &str) -> Self {
        self.no_guild_message = Some(content.to_owned());

        self
    }

    /// Message that's sent when user sends too few arguments to a command.
    ///
    /// %min% will be replaced with minimum allowed amount of arguments.
    ///
    /// %given% will be replced with the given amount of arguments.
    pub fn not_enough_args_message(mut self, content: &str) -> Self {
        self.not_enough_args_message = Some(content.to_owned());

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

        if let Ok(current_user) = rest::get_current_user() {
            self.on_mention = Some(vec![
                format!("<@{}>", current_user.id), // Regular mention
                format!("<@!{}>", current_user.id), // Nickname mention
            ]);
        }

        self
    }

    /// HashSet of user Ids checks won't apply to.
    pub fn owners(mut self, user_ids: HashSet<UserId>) -> Self {
        self.owners = user_ids;

        self
    }

    /// Sets the prefix to respond to. This can either be a single-char or
    /// multi-char string.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefixes = vec![prefix.to_owned()];

        self
    }

    /// Sets the prefixes to respond to. Those can either be single-chararacter or
    /// multi-chararacter strings.
    pub fn prefixes(mut self, prefixes: Vec<&str>) -> Self {
        self.prefixes = prefixes.iter().map(|x| x.to_string()).collect();

        self
    }

    /// Message that's sent when user sends too many arguments to a command.
    ///
    /// %max% will be replaced with maximum allowed amount of arguments.
    ///
    /// %given% will be replced with the given amount of arguments.
    pub fn too_many_args_message(mut self, content: &str) -> Self {
        self.too_many_args_message = Some(content.to_owned());

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
    fn default() -> Configuration {
        Configuration {
            depth: 5,
            on_mention: None,
            dynamic_prefix: None,
            allow_whitespace: false,
            prefixes: vec![],
            no_dm_message: None,
            no_guild_message: None,
            rate_limit_message: None,
            blocked_user_message: None,
            too_many_args_message: None,
            invalid_check_message: None,
            blocked_guild_message: None,
            not_enough_args_message: None,
            command_disabled_message: None,
            invalid_permission_message: None,
            account_type: AccountType::Automatic,
            owners: HashSet::default(),
            blocked_users: HashSet::default(),
            blocked_guilds: HashSet::default(),
            disabled_commands: HashSet::default(),
            allow_dm: true,
        }
    }
}
