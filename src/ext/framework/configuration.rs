use std::default::Default;
use ::client::rest;
use ::client::Context;

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
    pub dynamic_prefix: Option<Box<Fn(&Context) -> Option<String> + Send + Sync + 'static>>
}

impl Configuration {
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
    pub fn allow_whitespace(mut self, allow_whitespace: bool)
        -> Self {
        self.allow_whitespace = allow_whitespace;

        self
    }

    /// Sets the prefix to respond to. This can either be a single-char or
    /// multi-char string.
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefixes = vec![prefix.into()];

        self
    }

    /// Sets the prefix to respond to. This can either be a single-char or
    /// multi-char string.
    pub fn prefixes(mut self, prefixes: Vec<&str>) -> Self {
        self.prefixes = prefixes.iter().map(|x| x.to_string()).collect();

        self
    }

    /// Sets the prefix to respond to. This can either be a single-char or
    /// multi-char string.
    pub fn dynamic_prefix<F>(mut self, dynamic_prefix: F) -> Self
        where F: Fn(&Context) -> Option<String> + Send + Sync + 'static {
        self.dynamic_prefix = Some(Box::new(dynamic_prefix));

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
            allow_whitespace: false,
            prefixes: vec![],
            dynamic_prefix: None
        }
    }
}
