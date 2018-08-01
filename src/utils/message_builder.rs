use model::{
    guild::Emoji,
    id::{ChannelId, RoleId, UserId},
    misc::Mentionable
};
use std::{
    default::Default,
    fmt::{self, Display, Write},
    ops::Add
};

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
/// ```rust,no_run
/// # use serenity::model::prelude::*;
/// #
/// # let user = UserId(1);
/// # let emoji = Emoji {
/// #     animated: false,
/// #     id: EmojiId(2),
/// #     name: "test".to_string(),
/// #     managed: false,
/// #     require_colons: true,
/// #     roles: vec![],
/// # };
/// #
/// use serenity::utils::MessageBuilder;
///
/// // assuming an `emoji` and `user` have already been bound
///
/// let content = MessageBuilder::new()
///     .push("You sent a message, ")
///     .mention(&user)
///     .push("! ")
///     .mention(&emoji)
///     .build();
/// ```
///
/// [`build`]: #method.build
/// [`emoji`]: #method.emoji
/// [`user`]: #method.user
#[derive(Clone, Debug, Default)]
pub struct MessageBuilder(pub String);

impl MessageBuilder {
    /// Creates a new, empty builder.
    ///
    /// # Examples
    ///
    /// Create a new `MessageBuilder`:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let message = MessageBuilder::new();
    ///
    /// // alternatively:
    /// let message = MessageBuilder::default();
    /// ```
    pub fn new() -> MessageBuilder { MessageBuilder::default() }

    /// Pulls the inner value out of the builder.
    ///
    /// # Examples
    ///
    /// Create a string mentioning a channel by Id, and then suffixing `"!"`,
    /// and finally building it to retrieve the inner String:
    ///
    /// ```rust
    /// use serenity::model::id::ChannelId;
    /// use serenity::utils::MessageBuilder;
    ///
    /// let channel_id = ChannelId(81384788765712384);
    ///
    /// let content = MessageBuilder::new()
    ///     .channel(channel_id)
    ///     .push("!")
    ///     .build();
    ///
    /// assert_eq!(content, "<#81384788765712384>!");
    /// ```
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
    pub fn build(self) -> String { self.0 }

    /// Mentions the [`GuildChannel`] in the built message.
    ///
    /// This accepts anything that converts _into_ a [`ChannelId`]. Refer to
    /// `ChannelId`'s documentation for more information.
    ///
    /// Refer to `ChannelId`'s [Display implementation] for more information on
    /// how this is formatted.
    ///
    /// # Examples
    ///
    /// Mentioning a [`Channel`] by Id:
    ///
    /// ```rust
    /// use serenity::model::id::ChannelId;
    /// use serenity::utils::MessageBuilder;
    ///
    /// let channel_id = ChannelId(81384788765712384);
    ///
    /// let content = MessageBuilder::new()
    ///     .push("The channel is: ")
    ///     .channel(channel_id)
    ///     .build();
    ///
    /// assert_eq!(content, "The channel is: <#81384788765712384>");
    /// ```
    ///
    /// [`Channel`]: ../model/channel/enum.Channel.html
    /// [`ChannelId`]: ../model/id/struct.ChannelId.html
    /// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
    /// [Display implementation]: ../model/id/struct.ChannelId.html#method.fmt-1
    #[inline]
    pub fn channel<C: Into<ChannelId>>(self, channel: C) -> Self {
        self._channel(channel.into())
    }

    fn _channel(mut self, channel: ChannelId) -> Self {
        let _ = write!(self.0, "{}", channel.mention());

        self
    }

    /// Displays the given emoji in the built message.
    ///
    /// Refer to `Emoji`s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// # Examples
    ///
    /// Mention an emoji in a message's content:
    ///
    /// ```rust
    /// use serenity::model::guild::Emoji;
    /// use serenity::model::id::EmojiId;
    /// use serenity::utils::MessageBuilder;
    ///
    /// let emoji = Emoji {
    ///     animated: false,
    ///     id: EmojiId(302516740095606785),
    ///     managed: true,
    ///     name: "smugAnimeFace".to_string(),
    ///     require_colons: true,
    ///     roles: vec![],
    /// };
    ///
    /// let message = MessageBuilder::new()
    ///     .push("foo ")
    ///     .emoji(&emoji)
    ///     .push(".")
    ///     .build();
    ///
    /// assert_eq!(message, "foo <:smugAnimeFace:302516740095606785>.");
    /// ```
    ///
    /// [Display implementation]: ../model/guild/struct.Emoji.html#method.fmt
    pub fn emoji(mut self, emoji: &Emoji) -> Self {
        let _ = write!(self.0, "{}", emoji);

        self
    }

    /// Mentions something that implements the [`Mentionable`] trait.
    ///
    /// [`Mentionable`]: ../model/misc/trait.Mentionable.html
    pub fn mention<M: Mentionable>(mut self, item: &M) -> Self {
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
    #[inline]
    pub fn push<D: I>(self, content: D) -> Self {
        self._push(&content.into().to_string())
    }

    fn _push(mut self, content: &str) -> Self {
        self.0.push_str(content);

        self
    }

    /// Pushes a codeblock to the content, with optional syntax highlighting.
    ///
    /// # Examples
    ///
    /// Pushing a Rust codeblock:
    ///
    /// ```rust,ignore
    /// use serenity::utils::MessageBuilder;
    ///
    /// let code = r#"
    /// fn main() {
    ///     println!("Hello, world!");
    /// }
    /// "#;
    ///
    /// let content = MessageBuilder::new()
    ///     .push_codeblock(code, Some("rust"))
    ///     .build();
    ///
    /// let expected = r#"```rust
    /// fn main() {
    ///     println!("Hello, world!");
    /// }
    /// ```"#;
    ///
    /// assert_eq!(content, expected);
    /// ```
    ///
    /// Pushing a codeblock without a language:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///     .push_codeblock("hello", None)
    ///     .build();
    ///
    /// assert_eq!(content, "```\nhello\n```");
    /// ```
    pub fn push_codeblock<D: I>(mut self, content: D, language: Option<&str>) -> Self {
        self.0.push_str("```");

        if let Some(language) = language {
            self.0.push_str(language);
        }

        self.0.push('\n');
        self.0.push_str(&content.into().to_string());
        self.0.push_str("\n```");

        self
    }

    /// Pushes inlined monospaced text to the content.
    ///
    /// # Examples
    ///
    /// Display a server configuration value to the user:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let key = "prefix";
    /// let value = "&";
    ///
    /// let content = MessageBuilder::new()
    ///     .push("The setting ")
    ///     .push_mono(key)
    ///     .push(" for this server is ")
    ///     .push_mono(value)
    ///     .push(".")
    ///     .build();
    ///
    /// let expected = format!("The setting `{}` for this server is `{}`.",
    ///                        key,
    ///                        value);
    ///
    /// assert_eq!(content, expected);
    /// ```
    pub fn push_mono<D: I>(mut self, content: D) -> Self {
        self.0.push('`');
        self.0.push_str(&content.into().to_string());
        self.0.push('`');

        self
    }

    /// Pushes inlined italicized text to the content.
    ///
    /// # Examples
    ///
    /// Emphasize information to the user:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///     .push("You don't ")
    ///     .push_italic("always need")
    ///     .push(" to italicize ")
    ///     .push_italic("everything")
    ///     .push(".")
    ///     .build();
    ///
    /// let expected = "You don't _always need_ to italicize _everything_.";
    ///
    /// assert_eq!(content, expected);
    /// ```
    pub fn push_italic<D: I>(mut self, content: D) -> Self {
        self.0.push('_');
        self.0.push_str(&content.into().to_string());
        self.0.push('_');

        self
    }

    /// Pushes an inline bold text to the content.
    pub fn push_bold<D: I>(mut self, content: D) -> Self {
        self.0.push_str("**");
        self.0.push_str(&content.into().to_string());
        self.0.push_str("**");

        self
    }

    /// Pushes an underlined inline text to the content.
    pub fn push_underline<D: I>(mut self, content: D) -> Self {
        self.0.push_str("__");
        self.0.push_str(&content.into().to_string());
        self.0.push_str("__");

        self
    }

    /// Pushes a strikethrough inline text to the content.
    pub fn push_strike<D: I>(mut self, content: D) -> Self {
        self.0.push_str("~~");
        self.0.push_str(&content.into().to_string());
        self.0.push_str("~~");

        self
    }

    /// Pushes the given text with a newline appended to the content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "hello\nworld");
    /// ```
    pub fn push_line<D: I>(mut self, content: D) -> Self {
        self = self.push(content);
        self.0.push('\n');

        self
    }

    /// Pushes inlined monospace text with an added newline to the content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_mono_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "`hello`\nworld");
    /// ```
    pub fn push_mono_line<D: I>(mut self, content: D) -> Self {
        self = self.push_mono(content);
        self.0.push('\n');

        self
    }

    /// Pushes an inlined italicized text with an added newline to the content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_italic_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "_hello_\nworld");
    /// ```
    pub fn push_italic_line<D: I>(mut self, content: D) -> Self {
        self = self.push_italic(content);
        self.0.push('\n');

        self
    }

    /// Pushes an inline bold text with an added newline to the content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_bold_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "**hello**\nworld");
    /// ```
    pub fn push_bold_line<D: I>(mut self, content: D) -> Self {
        self = self.push_bold(content);
        self.0.push('\n');

        self
    }

    /// Pushes an underlined inline text with an added newline to the content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_underline_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "__hello__\nworld");
    /// ```
    pub fn push_underline_line<D: I>(mut self, content: D) -> Self {
        self = self.push_underline(content);
        self.0.push('\n');

        self
    }

    /// Pushes a strikethrough inline text with a newline added to the content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_strike_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "~~hello~~\nworld");
    /// ```
    pub fn push_strike_line<D: I>(mut self, content: D) -> Self {
        self = self.push_strike(content);
        self.0.push('\n');

        self
    }

    /// Pushes text to your message, but normalizing content - that means
    /// ensuring that there's no unwanted formatting, mention spam etc.
    pub fn push_safe<C: I>(mut self, content: C) -> Self {
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner)
                .replace('*', "\\*")
                .replace('`', "\\`")
                .replace('_', "\\_");

            self.0.push_str(&c.to_string());
        }

        self
    }

    /// Pushes a code-block to your message normalizing content.
    pub fn push_codeblock_safe<D: I>(mut self, content: D, language: Option<&str>) -> Self {
        self.0.push_str("```");

        if let Some(language) = language {
            self.0.push_str(language);
        }

        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("```", " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("```");

        self
    }

    /// Pushes an inline monospaced text to the content normalizing content.
    pub fn push_mono_safe<D: I>(mut self, content: D) -> Self {
        self.0.push('`');
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace('`', "'");
            self.0.push_str(&c.to_string());
        }
        self.0.push('`');

        self
    }

    /// Pushes an inline italicized text to the content normalizing content.
    pub fn push_italic_safe<D: I>(mut self, content: D) -> Self {
        self.0.push('_');
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace('_', " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push('_');

        self
    }

    /// Pushes an inline bold text to the content normalizing content.
    pub fn push_bold_safe<D: I>(mut self, content: D) -> Self {
        self.0.push_str("**");
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("**", " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("**");

        self
    }

    /// Pushes an underlined inline text to the content normalizing content.
    pub fn push_underline_safe<D: I>(mut self, content: D) -> Self {
        self.0.push_str("__");
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("__", " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("__");

        self
    }

    /// Pushes a strikethrough inline text to the content normalizing content.
    pub fn push_strike_safe<D: I>(mut self, content: D) -> Self {
        self.0.push_str("~~");
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("~~", " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("~~");

        self
    }

    /// Pushes text with a newline appended to the content normalizing content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_line_safe("Hello @everyone")
    ///                                    .push("How are you?")
    ///                                    .build();
    ///
    /// assert_eq!(content, "Hello @\u{200B}everyone\nHow are you?");
    /// ```
    pub fn push_line_safe<D: I>(mut self, content: D) -> Self {
        self = self.push_safe(content);
        self.0.push('\n');

        self
    }

    /// Pushes an inline monospaced text with added newline to the content normalizing content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///                 .push_mono_line_safe("`hello @everyone`")
    ///                 .push("world").build();
    ///
    /// assert_eq!(content, "`'hello @\u{200B}everyone'`\nworld");
    /// ```
    pub fn push_mono_line_safe<D: I>(mut self, content: D) -> Self {
        self = self.push_mono_safe(content);
        self.0.push('\n');

        self
    }

    /// Pushes an inline italicized text with added newline to the content normalizing content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///                 .push_italic_line_safe("@everyone")
    ///                 .push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "_@\u{200B}everyone_\nIsn't a mention.");
    /// ```
    pub fn push_italic_line_safe<D: I>(mut self, content: D) -> Self {
        self = self.push_italic_safe(content);
        self.0.push('\n');

        self
    }

    /// Pushes an inline bold text with added newline to the content normalizing content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///                 .push_bold_line_safe("@everyone")
    ///                 .push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "**@\u{200B}everyone**\nIsn't a mention.");
    /// ```
    pub fn push_bold_line_safe<D: I>(mut self, content: D) -> Self {
        self = self.push_bold_safe(content);
        self.0.push('\n');

        self
    }

    /// Pushes an underlined inline text with added newline to the content normalizing content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///                 .push_underline_line_safe("@everyone")
    ///                 .push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "__@\u{200B}everyone__\nIsn't a mention.");
    /// ```
    pub fn push_underline_line_safe<D: I>(mut self, content: D) -> Self {
        self = self.push_underline_safe(content);
        self.0.push('\n');

        self
    }

    /// Pushes a strikethrough inline text with added newline to the content normalizing
    /// content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///                 .push_strike_line_safe("@everyone")
    ///                 .push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "~~@\u{200B}everyone~~\nIsn't a mention.");
    /// ```
    pub fn push_strike_line_safe<D: I>(mut self, content: D) -> Self {
        self = self.push_strike_safe(content);
        self.0.push('\n');

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
    /// [`Role`]: ../model/guild/struct.Role.html
    /// [`RoleId`]: ../model/id/struct.RoleId.html
    /// [Display implementation]: ../model/id/struct.RoleId.html#method.fmt-1
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
    /// [`User`]: ../model/user/struct.User.html
    /// [`UserId`]: ../model/id/struct.UserId.html
    /// [Display implementation]: ../model/id/struct.UserId.html#method.fmt-1
    pub fn user<U: Into<UserId>>(mut self, user: U) -> Self {
        let _ = write!(self.0, "{}", user.into().mention());

        self
    }
}

impl Display for MessageBuilder {
    /// Formats the message builder into a string.
    ///
    /// This is done by simply taking the internal value of the tuple-struct and
    /// writing it into the formatter.
    ///
    /// # Examples
    ///
    /// Create a message builder, and format it into a string via the `format!`
    /// macro:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    ///
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}


/// Formatting modifiers for MessageBuilder content pushes
///
/// Provides an enum of formatting modifiers for a string, for combination with
/// string types and Content types.
///
/// # Examples
///
/// Create a new Content type which describes a bold-italic "text":
///
/// ```rust,no_run
/// use serenity::utils::ContentModifier::{Bold, Italic};
/// use serenity::utils::Content;
/// let content: Content = Bold + Italic + "text";
/// ```
pub enum ContentModifier {
    Italic,
    Bold,
    Strikethrough,
    Code,
    Underline,
}

/// Describes formatting on string content
#[derive(Debug, Default, Clone)]
pub struct Content {
    pub italic: bool,
    pub bold: bool,
    pub strikethrough: bool,
    pub inner: String,
    pub code: bool,
    pub underline: bool,
}

impl<T: ToString> Add<T> for Content {
    type Output = Content;

    fn add(mut self, rhs: T) -> Content {
        self.inner = self.inner + &rhs.to_string();

        self
    }
}

impl<T: ToString> Add<T> for ContentModifier {
    type Output = Content;

    fn add(self, rhs: T) -> Content {
        let mut nc = self.to_content();
        nc.inner = nc.inner + &rhs.to_string();

        nc
    }
}

impl Add<ContentModifier> for Content {
    type Output = Content;

    fn add(mut self, rhs: ContentModifier) -> Content {
        self.apply(&rhs);

        self
    }
}

impl Add<ContentModifier> for ContentModifier {
    type Output = Content;

    fn add(self, rhs: ContentModifier) -> Content {
        let mut nc = self.to_content();
        nc.apply(&rhs);

        nc
    }
}

impl ContentModifier {
    fn to_content(&self) -> Content {
        let mut nc = Content::default();
        nc.apply(self);

        nc
    }
}

impl Content {
    pub fn apply(&mut self, modifier: &ContentModifier) {
        match *modifier {
            ContentModifier::Italic => {
                self.italic = true;
            },
            ContentModifier::Bold => {
                self.bold = true;
            },
            ContentModifier::Strikethrough => {
                self.strikethrough = true;
            },
            ContentModifier::Code => {
                self.code = true;
            },
            ContentModifier::Underline => {
                self.underline = true;
            },
        }
    }

    pub fn to_string(&self) -> String {
        let capacity =
            self.inner.len() + if self.bold { 4 } else { 0 } + if self.italic { 2 } else { 0 } +
            if self.strikethrough { 4 } else { 0 } + if self.underline {
                4
            } else {
                0
            } + if self.code { 2 } else { 0 };
        let mut new_str = String::with_capacity(capacity);

        if self.bold {
            new_str.push_str("**");
        }

        if self.italic {
            new_str.push('*');
        }

        if self.strikethrough {
            new_str.push_str("~~");
        }

        if self.underline {
            new_str.push_str("__");
        }

        if self.code {
            new_str.push('`');
        }

        new_str.push_str(&self.inner);

        if self.code {
            new_str.push('`');
        }

        if self.underline {
            new_str.push_str("__");
        }

        if self.strikethrough {
            new_str.push_str("~~");
        }

        if self.italic {
            new_str.push('*');
        }

        if self.bold {
            new_str.push_str("**");
        }

        new_str
    }
}

impl From<ContentModifier> for Content {
    fn from(cm: ContentModifier) -> Content { cm.to_content() }
}

mod private {
    use super::{Content, ContentModifier};
    use std::fmt;

    pub trait A {}

    impl A for ContentModifier {}
    impl A for Content {}
    impl<T: fmt::Display> A for T {}
}


/// This trait only exists as way to bypass the shouting of the compiler. Specifically "conflicting
/// implementations in core" and alike.
/// However is not meant to be used outside.
pub trait I: self::private::A {
    fn into(self) -> Content;
}

impl<T: fmt::Display> I for T {
    fn into(self) -> Content {
        Content {
            italic: false,
            bold: false,
            strikethrough: false,
            inner: self.to_string(),
            code: false,
            underline: false,
        }
    }
}

impl I for ContentModifier {
    fn into(self) -> Content { self.to_content() }
}

impl I for Content {
    fn into(self) -> Content { self }
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

#[cfg(test)]
mod test {
    use model::prelude::*;
    use super::{
        ContentModifier::*,
        MessageBuilder,
    };

    #[test]
    fn code_blocks() {
        let content = MessageBuilder::new()
            .push_codeblock("test", Some("rb"))
            .build();
        assert_eq!(content, "```rb\ntest\n```");
    }

    #[test]
    fn safe_content() {
        let content = MessageBuilder::new()
            .push_safe("@everyone discord.gg/discord-api")
            .build();
        assert_ne!(content, "@everyone discord.gg/discord-api");
    }

    #[test]
    fn no_free_formatting() {
        let content = MessageBuilder::new().push_bold_safe("test**test").build();
        assert_ne!(content, "**test**test**");
    }

    #[test]
    fn mentions() {
        let content_emoji = MessageBuilder::new()
            .emoji(&Emoji {
                animated: false,
                id: EmojiId(32),
                name: "Rohrkatze".to_string(),
                managed: false,
                require_colons: true,
                roles: vec![],
            })
            .build();
        let content_mentions = MessageBuilder::new()
            .channel(1)
            .mention(&UserId(2))
            .role(3)
            .user(4)
            .build();
        assert_eq!(content_mentions, "<#1><@2><@&3><@4>");
        assert_eq!(content_emoji, "<:Rohrkatze:32>");
    }

    #[test]
    fn content() {
        let content = Bold + Italic + Code + "Fun!";

        assert_eq!(content.to_string(), "***`Fun!`***");
    }

    #[test]
    fn message_content() {
        let message_content = MessageBuilder::new()
            .push(Bold + Italic + Code + "Fun!")
            .build();

        assert_eq!(message_content, "***`Fun!`***");
    }

    #[test]
    fn message_content_safe() {
        let message_content = MessageBuilder::new()
            .push_safe(Bold + Italic + "test**test")
            .build();

        assert_eq!(message_content, "***test\\*\\*test***");
    }
}
