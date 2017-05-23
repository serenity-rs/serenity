use std::default::Default;
use std::fmt::{self, Write};
use ::model::{ChannelId, Emoji, Mentionable, RoleId, UserId};

/// The Message Builder is an ergonomic utility to easily build a message,
/// by adding text and mentioning mentionable structs.
///
/// The finalized value can be accessed via [`build`] or the inner value.
///
/// # Examples
///
/// Build a message, mentioning a [`user`] and an [`emoji`], and retrieving the
/// value:
///
/// ```rust,ignore
/// use serenity::utils::MessageBuilder;
///
/// // assuming an `emoji` and `user` have already been bound
///
/// let content = MessageBuilder::new()
///     .push("You sent a message, ")
///     .mention(user)
///     .push("! ")
///     .mention(emoji)
///     .build();
/// ```
///
/// [`build`]: #method.build
/// [`emoji`]: #method.emoji
/// [`user`]: #method.user
#[derive(Clone, Debug, Default)]
pub struct MessageBuilder(pub String);

impl MessageBuilder {
    /// Creates a new, empty-content builder.
    pub fn new() -> MessageBuilder {
        MessageBuilder::default()
    }

    /// Pulls the inner value out of the builder.
    ///
    /// # Examples
    ///
    /// This is equivalent to simply retrieving the tuple struct's first value:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push("test").0;
    ///
    /// assert_eq!(content, "test");
    /// ```
    pub fn build(self) -> String {
        self.0
    }

    /// Mentions the [`GuildChannel`] in the built message.
    ///
    /// This accepts anything that converts _into_ a [`ChannelId`]. Refer to
    /// `ChannelId`'s documentation for more information.
    ///
    /// Refer to `ChannelId`'s [Display implementation] for more information on
    /// how this is formatted.
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`GuildChannel`]: ../model/struct.GuildChannel.html
    /// [Display implementation]: ../model/struct.ChannelId.html#method.fmt-1
    pub fn channel<C: Into<ChannelId>>(mut self, channel: C) -> Self {
        let _ = write!(self.0, "{}", channel.into().mention());

        self
    }

    /// Displays the given emoji in the built message.
    ///
    /// Refer to `Emoji`s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// [Display implementation]: ../model/struct.Emoji.html#method.fmt
    pub fn emoji(mut self, emoji: Emoji) -> Self {
        let _ = write!(self.0, "{}", emoji);

        self
    }

    /// Mentions something that implements the [`Mentionable`] trait.
    ///
    /// [`Mentionable`]: ../model/trait.Mentionable.html
    pub fn mention<M: Mentionable>(mut self, item: M) -> Self {
        let _ = write!(self.0, "{}", item.mention());

        self
    }

    /// Pushes a string to the internal message content.
    ///
    /// Note that this does not mutate either the given data or the internal
    /// message content in anyway prior to appending the given content to the
    /// internal message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let message = MessageBuilder::new().push("test");
    ///
    /// assert_eq!(message.push("ing").0, "testing");
    /// ```
    pub fn push(mut self, content: &str) -> Self {
        self.0.push_str(content);

        self
    }

    /// Pushes a code-block to your message, with optional syntax highlighting.
    pub fn push_codeblock(mut self, content: &str, language: Option<&str>) -> Self {
        self.0.push_str("```");

        if let Some(language) = language {
            self.0.push_str(language);
        }

        self.0.push('\n');
        self.0.push_str(content);
        self.0.push_str("\n```");

        self
    }

    /// Pushes an inline monospaced text to your message.
    pub fn push_mono(mut self, content: &str) -> Self {
        self.0.push('`');
        self.0.push_str(content);
        self.0.push('`');

        self
    }

    /// Pushes an inline italicized text to your message.
    pub fn push_italic(mut self, content: &str) -> Self {
        self.0.push('_');
        self.0.push_str(content);
        self.0.push('_');

        self
    }

    /// Pushes an inline bold text to your message.
    pub fn push_bold(mut self, content: &str) -> Self {
        self.0.push_str("**");
        self.0.push_str(content);
        self.0.push_str("**");

        self
    }

    /// Pushes an underlined inline text to your message.
    pub fn push_underline(mut self, content: &str) -> Self {
        self.0.push_str("__");
        self.0.push_str(content);
        self.0.push_str("__");

        self
    }

    /// Pushes a strikethrough inline text to your message.
    pub fn push_strike(mut self, content: &str) -> Self {
        self.0.push_str("~~");
        self.0.push_str(content);
        self.0.push_str("~~");

        self
    }

    /// Pushes text to your message, but normalizing content - that means
    /// ensuring that there's no unwanted formatting, mention spam etc.
    pub fn push_safe(mut self, content: &str) -> Self {
        let normalized = normalize(content)
            .replace('*', "\\*")
            .replace('`', "\\`")
            .replace('_', "\\_");

        self.0.push_str(&normalized);

        self
    }

    /// Pushes a code-block to your message normalizing content.
    pub fn push_codeblock_safe(mut self, content: &str, language: Option<&str>)
        -> Self {
        let content = &normalize(content)
            .replace("```", "'''");

        self.0.push_str("```");

        if let Some(language) = language {
            self.0.push_str(language);
        }

        self.0.push('\n');
        self.0.push_str(content);
        self.0.push_str("```");

        self
    }

    /// Pushes an inline monospaced text to your message normalizing content.
    pub fn push_mono_safe(mut self, content: &str) -> Self {
        self.0.push('`');
        self.0.push_str(&normalize(content).replace('`', "'"));
        self.0.push('`');

        self
    }

    /// Pushes an inline italicized text to your message normalizing content.
    pub fn push_italic_safe(mut self, content: &str) -> Self {
        self.0.push('_');
        self.0.push_str(&normalize(content).replace('_', " "));
        self.0.push('_');

        self
    }

    /// Pushes an inline bold text to your message normalizing content.
    pub fn push_bold_safe(mut self, content: &str) -> Self {
        self.0.push_str("**");
        self.0.push_str(&normalize(content).replace("**", "  "));
        self.0.push_str("**");

        self
    }

    /// Pushes an underlined inline text to your message normalizing content.
    pub fn push_underline_safe(mut self, content: &str) -> Self {
        self.0.push_str("__");
        self.0.push_str(&normalize(content).replace("__", "  "));
        self.0.push_str("__");

        self
    }

    /// Pushes a strikethrough inline text to your message normalizing content.
    pub fn push_strike_safe(mut self, content: &str) -> Self {
        self.0.push_str("~~");
        self.0.push_str(&normalize(content).replace("~~", "  "));
        self.0.push_str("~~");

        self
    }

    /// Mentions the [`Role`] in the built message.
    ///
    /// This accepts anything that converts _into_ a [`RoleId`]. Refer to
    /// `RoleId`'s documentation for more information.
    ///
    /// Refer to `RoleId`'s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// [`Role`]: ../model/struct.Role.html
    /// [`RoleId`]: ../model/struct.RoleId.html
    /// [Display implementation]: ../model/struct.RoleId.html#method.fmt-1
    pub fn role<R: Into<RoleId>>(mut self, role: R) -> Self {
        let _ = write!(self.0, "{}", role.into().mention());

        self
    }

    /// Mentions the [`User`] in the built message.
    ///
    /// This accepts anything that converts _into_ a [`UserId`]. Refer to
    /// `UserId`'s documentation for more information.
    ///
    /// Refer to `UserId`'s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// [`User`]: ../model/struct.User.html
    /// [`UserId`]: ../model/struct.UserId.html
    /// [Display implementation]: ../model/struct.UserId.html#method.fmt-1
    pub fn user<U: Into<UserId>>(mut self, user: U) -> Self {
        let _ = write!(self.0, "{}", user.into().mention());

        self
    }
}

impl fmt::Display for MessageBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

fn normalize(text: &str) -> String {
    // Remove invite links and popular scam websites, mostly to prevent the
    // current user from triggering various ad detectors and prevent embeds.
    text.replace("discord.gg", "discord\u{2024}gg")
        .replace("discord.me", "discord\u{2024}me")
        .replace("discordlist.net", "discordlist\u{2024}net")
        .replace("discordservers.com", "discordservers\u{2024}com")
        .replace("discordapp.com/invite", "discordapp\u{2024}com/invite")
        // Remove right-to-left override and other similar annoying symbols
        .replace('\u{202E}', " ") // RTL Override
        .replace('\u{200F}', " ") // RTL Mark
        .replace('\u{202B}', " ") // RTL Embedding
        .replace('\u{200B}', " ") // Zero-width space
        .replace('\u{200D}', " ") // Zero-width joiner
        .replace('\u{200C}', " ") // Zero-width non-joiner
        // Remove everyone and here mentions. Has to be put after ZWS replacement
        // because it utilises it itself.
        .replace("@everyone", "@\u{200B}everyone")
        .replace("@here", "@\u{200B}here")
}
