use std::default::Default;
use std::fmt;
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

fn normalize(text: &str) -> String {
    // Remove everyone and here mentions
    // This changes 'at' symbol to a full-width variation
    let mut new_text = text.replace("@everyone", "＠everyone")
        .replace("@here", "＠here")
    // Remove invite links and popular scam websites
    // mostly to prevent our bot triggering various ads detectors
        .replace("discord.gg", "discord․gg")
        .replace("discord.me", "discord․me")
        .replace("discordlist.net", "discordlist․net")
        .replace("discordservers.com", "discordlist․net")
        .replace("discordapp.com/invite", "discordapp․com/invite")
    // Remove right-to-left override and similar
        .replace("\u{202E}", " ")  // RTL
        .replace("\u{200F}", " ")  // RTL Mark
        .replace("\u{202B}", " ")  // RTL Embedding
        .replace("\u{200B}", " ")  // Zero-width space
        .replace("\u{200D}", " ")  // Zero-width joiner
        .replace("\u{200C}", " "); // Zero-width non-joiner

    // I'm going quite a bit lazy with this, but at least
    // we don't have to fetch members.
    if new_text.split("<@").count() > 3 {
        new_text = new_text.replace("<@!", "<user ")
            .replace("<@", "<user ");
    }
    if new_text.split("<&").count() > 3 {
        new_text = new_text.replace("<&", "<role ");
    }

    new_text
}

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
    /// This is equivilant to simply retrieving the tuple struct's first value:
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
        self.0.push_str(&format!("{}", channel.into()));

        self
    }

    /// Displays the given emoji in the built message.
    ///
    /// Refer to `Emoji`s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// [Display implementation]: ../model/struct.Emoji.html#method.fmt
    pub fn emoji(mut self, emoji: Emoji) -> Self {
        self.0.push_str(&format!("{}", emoji));

        self
    }

    /// Mentions something that implements the [`Mentionable`] trait.
    ///
    /// [`Mentionable`]: ../model/trait.Mentionable.html
    pub fn mention<M: Mentionable>(mut self, item: M) -> Self {
        self.0.push_str(&item.mention());

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
        match language {
            Some(x) => {
                self.0.push_str(&format!("```{}\n{}\n```", x, content));
            },
            None => {
                self.0.push_str(&format!("```\n{}\n```", content));
            }
        }

        self
    }

    /// Pushes an inline monospaced text to your message.
    pub fn push_mono(mut self, content: &str) -> Self {
        self.0.push_str(&format!("`{}`", content));

        self
    }

    /// Pushes an inline italicized text to your message.
    pub fn push_italic(mut self, content: &str) -> Self {
        self.0.push_str(&format!("_{}_", content));

        self
    }

    /// Pushes an inline bold text to your message.
    pub fn push_bold(mut self, content: &str) -> Self {
        self.0.push_str(&format!("**{}**", content));

        self
    }

    /// Pushes an underlined inline text to your message.
    pub fn push_underline(mut self, content: &str) -> Self {
        self.0.push_str(&format!("__{}__", content));

        self
    }

    /// Pushes a strikethrough inline text to your message.
    pub fn push_strike(mut self, content: &str) -> Self {
        self.0.push_str(&format!("~~{}~~", content));

        self
    }

    /// Pushes text to your message, but normalizing content - that means
    /// ensuring that there's no unwanted formatting, mention spam etc.
    pub fn push_safe(mut self, content: &str) -> Self {
        self.0.push_str(
            &normalize(&content).replace("*", "\\*")
                .replace("`", "\\`")
                .replace("_", "\\_")
        );

        self
    }

    /// Pushes a code-block to your message normalizing content.
    pub fn push_codeblock_safe(mut self, ct: &str, language: Option<&str>) -> Self {
        let content = &normalize(&ct)
            .replace("```", "\u{201B}\u{201B}\u{201B}");

        match language {
            Some(x) => {
                self.0.push_str(&format!("```{}\n{}\n```", x, content));
            },
            None => {
                self.0.push_str(&format!("```\n{}\n```", content));
            }
        }

        self
    }

    /// Pushes an inline monospaced text to your message normalizing content.
    pub fn push_mono_safe(mut self, content: &str) -> Self {
        self.0.push_str(
            &format!(
                "`{}`",
                &normalize(&content).replace("`", "\u{201B}")
            )
        );

        self
    }

    /// Pushes an inline italicized text to your message normalizing content.
    pub fn push_italic_safe(mut self, content: &str) -> Self {
        self.0.push_str(
            &format!(
                "_{}_",
                &normalize(&content).replace("_", "＿")
            )
        );

        self
    }

    /// Pushes an inline bold text to your message normalizing content.
    pub fn push_bold_safe(mut self, content: &str) -> Self {
        self.0.push_str(
            &format!(
                "**{}**",
                &normalize(&content).replace("**", "∗∗")
            )
        );

        self
    }

    /// Pushes an underlined inline text to your message normalizing content.
    pub fn push_underline_safe(mut self, content: &str) -> Self {
        self.0.push_str(
            &format!(
                "__{}__",
                &normalize(&content).replace("__", "＿＿")
            )
        );

        self
    }

    /// Pushes a strikethrough inline text to your message normalizing content.
    pub fn push_strike_safe(mut self, content: &str) -> Self {
        self.0.push_str(
            &format!(
                "~~{}~~",
                &normalize(&content).replace("~~", "∼∼")
            )
        );

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
        self.0.push_str(&format!("{}", role.into()));

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
        self.0.push_str(&format!("{}", user.into()));

        self
    }
}

impl fmt::Display for MessageBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Default for MessageBuilder {
    fn default() -> MessageBuilder {
        MessageBuilder(String::default())
    }
}
