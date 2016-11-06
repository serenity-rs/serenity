use std::default::Default;
use ::client::http;

pub struct Configuration {
    pub depth: usize,
    pub on_mention: Option<Vec<String>>,
    pub allow_whitespace: bool,
    pub prefix: Option<String>,
}

impl Configuration {
    /// The default depth of the message to check for commands. Defaults to 5.
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = depth as usize;

        self
    }

    pub fn on_mention(mut self, on_mention: bool) -> Self {
        if !on_mention {
            return self;
        }

        if let Ok(current_user) = http::get_current_user() {
            self.on_mention = Some(vec![
                format!("<@{}>", current_user.id), // Regular mention
                format!("<@!{}>", current_user.id), // Nickname mention
            ]);
        }

        self
    }

    /// Whether to allow whitespace being optional between a mention and a
    /// command.
    ///
    /// **Note**: Defaults to `false`.
    ///
    /// # Examples
    ///
    /// Setting this to `true` will allow this scenario to occur, while `false`
    /// will not:
    ///
    /// ```ignore
    /// <@BOT_ID>about
    ///
    /// // bot processes and executes the "about" command if it exists
    /// ```
    pub fn allow_whitespace(mut self, allow_whitespace: bool)
        -> Self {
        self.allow_whitespace = allow_whitespace;

        self
    }

    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());

        self
    }
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            depth: 5,
            on_mention: None,
            allow_whitespace: false,
            prefix: None,
        }
    }
}
