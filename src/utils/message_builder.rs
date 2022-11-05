use std::fmt::{self as fmt, Write};
use std::ops::Add;

use crate::model::guild::Emoji;
use crate::model::id::{ChannelId, RoleId, UserId};
use crate::model::mention::Mentionable;

/// The Message Builder is an ergonomic utility to easily build a message,
/// by adding text and mentioning mentionable structs.
///
/// The finalized value can be accessed via [`Self::build`] or the inner value.
///
/// # Examples
///
/// Build a message, mentioning a [`Self::user`] and an [`Self::emoji`], and retrieving the
/// value:
///
/// ```rust,no_run
/// # use serenity::model::prelude::*;
/// #
/// # fn run(user: UserId, emoji: Emoji) {
/// #
/// use serenity::utils::MessageBuilder;
///
/// // assuming an `emoji` and `user` have already been bound
///
/// let content = MessageBuilder::new()
///     .push("You sent a message, ")
///     .mention(&user)
///     .push("! ")
///     .emoji(&emoji)
///     .build();
/// # }
/// ```
#[derive(Clone, Debug, Default)]
pub struct MessageBuilder(pub String);

impl MessageBuilder {
    /// Creates a new, empty builder.
    ///
    /// # Examples
    ///
    /// Create a new [`MessageBuilder`]:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let message = MessageBuilder::new();
    ///
    /// // alternatively:
    /// let message = MessageBuilder::default();
    /// ```
    #[must_use]
    pub fn new() -> MessageBuilder {
        MessageBuilder::default()
    }

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
    /// let content = MessageBuilder::new().channel(channel_id).push("!").build();
    ///
    /// assert_eq!(content, "<#81384788765712384>!");
    /// ```
    ///
    /// This is equivalent to simply retrieving the tuple struct's first value:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let mut content = MessageBuilder::new();
    /// content.push("test");
    ///
    /// assert_eq!(content.build(), "test");
    /// ```
    pub fn build(&mut self) -> String {
        self.clone().0
    }

    /// Mentions the [`GuildChannel`] in the built message.
    ///
    /// This accepts anything that converts _into_ a [`ChannelId`]. Refer to
    /// [`ChannelId`]'s documentation for more information.
    ///
    /// Refer to [`ChannelId`]'s [Display implementation] for more information on
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
    /// let content = MessageBuilder::new().push("The channel is: ").channel(channel_id).build();
    ///
    /// assert_eq!(content, "The channel is: <#81384788765712384>");
    /// ```
    ///
    /// [`Channel`]: crate::model::channel::Channel
    /// [`GuildChannel`]: crate::model::channel::GuildChannel
    /// [Display implementation]: ChannelId#impl-Display
    #[inline]
    pub fn channel<C: Into<ChannelId>>(&mut self, channel: C) -> &mut Self {
        self._channel(channel.into())
    }

    fn _channel(&mut self, channel: ChannelId) -> &mut Self {
        self.0.push_str(&channel.mention().to_string());
        self
    }

    /// Displays the given emoji in the built message.
    ///
    /// Refer to [`Emoji`]s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// # Examples
    ///
    /// Mention an emoji in a message's content:
    ///
    /// ```rust
    /// # use serde_json::{json, from_value};
    /// # use serenity::model::guild::Emoji;
    /// # use serenity::model::id::EmojiId;
    /// # use serenity::utils::MessageBuilder;
    ///
    /// # let emoji = from_value::<Emoji>(json!({
    /// #     "id": EmojiId(302516740095606785),
    /// #     "name": "smugAnimeFace",
    /// # })).unwrap();
    ///
    /// let message = MessageBuilder::new().push("foo ").emoji(&emoji).push(".").build();
    ///
    /// assert_eq!(message, "foo <:smugAnimeFace:302516740095606785>.");
    /// ```
    ///
    /// [Display implementation]: crate::model::guild::Emoji#impl-Display
    pub fn emoji(&mut self, emoji: &Emoji) -> &mut Self {
        self.0.push_str(&emoji.to_string());
        self
    }

    /// Mentions something that implements the [`Mentionable`] trait.
    pub fn mention<M: Mentionable>(&mut self, item: &M) -> &mut Self {
        self.0.push_str(&item.mention().to_string());
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
    /// let mut message = MessageBuilder::new();
    /// message.push("test");
    ///
    /// assert_eq!(
    ///     {
    ///         message.push("ing");
    ///         message.build()
    ///     },
    ///     "testing"
    /// );
    /// ```
    #[inline]
    pub fn push(&mut self, content: impl Into<Content>) -> &mut Self {
        self._push(&content.into().to_string())
    }

    fn _push(&mut self, content: &str) -> &mut Self {
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
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new()
    ///     .push_codeblock("hello", None)
    ///     .build();
    ///
    /// assert_eq!(content, "```\nhello\n```");
    /// ```
    pub fn push_codeblock(
        &mut self,
        content: impl Into<Content>,
        language: Option<&str>,
    ) -> &mut Self {
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
    /// let expected = format!("The setting `{}` for this server is `{}`.", key, value);
    ///
    /// assert_eq!(content, expected);
    /// ```
    pub fn push_mono(&mut self, content: impl Into<Content>) -> &mut Self {
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
    pub fn push_italic(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push('_');
        self.0.push_str(&content.into().to_string());
        self.0.push('_');

        self
    }

    /// Pushes an inline bold text to the content.
    pub fn push_bold(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("**");
        self.0.push_str(&content.into().to_string());
        self.0.push_str("**");

        self
    }

    /// Pushes an underlined inline text to the content.
    pub fn push_underline(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("__");
        self.0.push_str(&content.into().to_string());
        self.0.push_str("__");

        self
    }

    /// Pushes a strikethrough inline text to the content.
    pub fn push_strike(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("~~");
        self.0.push_str(&content.into().to_string());
        self.0.push_str("~~");

        self
    }

    /// Pushes a spoiler'd inline text to the content.
    pub fn push_spoiler(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("||");
        self.0.push_str(&content.into().to_string());
        self.0.push_str("||");

        self
    }

    /// Pushes a quoted inline text to the content
    pub fn push_quote(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("> ");
        self.0.push_str(&content.into().to_string());

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
    pub fn push_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push(content);
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
    pub fn push_mono_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_mono(content);
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
    pub fn push_italic_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_italic(content);
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
    pub fn push_bold_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_bold(content);
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
    pub fn push_underline_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_underline(content);
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
    pub fn push_strike_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_strike(content);
        self.0.push('\n');

        self
    }

    /// Pushes a spoiler'd inline text with a newline added to the content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_spoiler_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "||hello||\nworld");
    /// ```
    pub fn push_spoiler_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_spoiler(content);
        self.0.push('\n');

        self
    }

    /// Pushes a quoted inline text to the content
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content = MessageBuilder::new().push_quote_line("hello").push("world").build();
    ///
    /// assert_eq!(content, "> hello\nworld");
    /// ```
    pub fn push_quote_line(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_quote(content);
        self.0.push('\n');

        self
    }

    /// Pushes text to your message, but normalizing content - that means
    /// ensuring that there's no unwanted formatting, mention spam etc.
    pub fn push_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        {
            let mut c = content.into();
            c.inner =
                normalize(&c.inner).replace('*', "\\*").replace('`', "\\`").replace('_', "\\_");

            self.0.push_str(&c.to_string());
        }

        self
    }

    /// Pushes a code-block to your message normalizing content.
    pub fn push_codeblock_safe(
        &mut self,
        content: impl Into<Content>,
        language: Option<&str>,
    ) -> &mut Self {
        self.0.push_str("```");

        if let Some(language) = language {
            self.0.push_str(language);
        }

        self.0.push('\n');
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("```", " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("\n```");

        self
    }

    /// Pushes an inline monospaced text to the content normalizing content.
    pub fn push_mono_safe(&mut self, content: impl Into<Content>) -> &mut Self {
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
    pub fn push_italic_safe(&mut self, content: impl Into<Content>) -> &mut Self {
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
    pub fn push_bold_safe(&mut self, content: impl Into<Content>) -> &mut Self {
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
    pub fn push_underline_safe(&mut self, content: impl Into<Content>) -> &mut Self {
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
    pub fn push_strike_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("~~");
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("~~", " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("~~");

        self
    }

    /// Pushes a spoiler'd inline text to the content normalizing content.
    pub fn push_spoiler_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("||");
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("||", " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("||");

        self
    }

    /// Pushes a quoted inline text to the content normalizing content.
    pub fn push_quote_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.0.push_str("> ");
        {
            let mut c = content.into();
            c.inner = normalize(&c.inner).replace("> ", " ");
            self.0.push_str(&c.to_string());
        }

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
    /// let content =
    ///     MessageBuilder::new().push_line_safe("Hello @everyone").push("How are you?").build();
    ///
    /// assert_eq!(content, "Hello @\u{200B}everyone\nHow are you?");
    /// ```
    pub fn push_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_safe(content);
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
    /// let content =
    ///     MessageBuilder::new().push_mono_line_safe("`hello @everyone`").push("world").build();
    ///
    /// assert_eq!(content, "`'hello @\u{200B}everyone'`\nworld");
    /// ```
    pub fn push_mono_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_mono_safe(content);
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
    /// let content =
    ///     MessageBuilder::new().push_italic_line_safe("@everyone").push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "_@\u{200B}everyone_\nIsn't a mention.");
    /// ```
    pub fn push_italic_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_italic_safe(content);
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
    /// let content =
    ///     MessageBuilder::new().push_bold_line_safe("@everyone").push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "**@\u{200B}everyone**\nIsn't a mention.");
    /// ```
    pub fn push_bold_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_bold_safe(content);
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
    ///     .push_underline_line_safe("@everyone")
    ///     .push("Isn't a mention.")
    ///     .build();
    ///
    /// assert_eq!(content, "__@\u{200B}everyone__\nIsn't a mention.");
    /// ```
    pub fn push_underline_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_underline_safe(content);
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
    /// let content =
    ///     MessageBuilder::new().push_strike_line_safe("@everyone").push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "~~@\u{200B}everyone~~\nIsn't a mention.");
    /// ```
    pub fn push_strike_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_strike_safe(content);
        self.0.push('\n');

        self
    }

    /// Pushes a spoiler'd inline text with added newline to the content normalizing
    /// content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content =
    ///     MessageBuilder::new().push_spoiler_line_safe("@everyone").push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "||@\u{200B}everyone||\nIsn't a mention.");
    /// ```
    pub fn push_spoiler_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_spoiler_safe(content);
        self.0.push('\n');

        self
    }

    /// Pushes a quoted inline text with added newline to the content normalizing
    /// content.
    ///
    /// # Examples
    ///
    /// Push content and then append a newline:
    ///
    /// ```rust
    /// use serenity::utils::MessageBuilder;
    ///
    /// let content =
    ///     MessageBuilder::new().push_quote_line_safe("@everyone").push("Isn't a mention.").build();
    ///
    /// assert_eq!(content, "> @\u{200B}everyone\nIsn't a mention.");
    /// ```
    pub fn push_quote_line_safe(&mut self, content: impl Into<Content>) -> &mut Self {
        self.push_quote_safe(content);
        self.0.push('\n');

        self
    }

    /// Starts a multi-line quote, every push after this one will be quoted
    pub fn quote_rest(&mut self) -> &mut Self {
        self.0.push_str("\n>>> ");

        self
    }

    /// Mentions the [`Role`] in the built message.
    ///
    /// This accepts anything that converts _into_ a [`RoleId`]. Refer to
    /// [`RoleId`]'s documentation for more information.
    ///
    /// Refer to [`RoleId`]'s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// [`Role`]: crate::model::guild::Role
    /// [Display implementation]: RoleId#impl-Display
    pub fn role<R: Into<RoleId>>(&mut self, role: R) -> &mut Self {
        self.0.push_str(&role.into().mention().to_string());
        self
    }

    /// Mentions the [`User`] in the built message.
    ///
    /// This accepts anything that converts _into_ a [`UserId`]. Refer to
    /// [`UserId`]'s documentation for more information.
    ///
    /// Refer to [`UserId`]'s [Display implementation] for more information on how
    /// this is formatted.
    ///
    /// [`User`]: crate::model::user::User
    /// [Display implementation]: UserId#impl-Display
    pub fn user<U: Into<UserId>>(&mut self, user: U) -> &mut Self {
        self.0.push_str(&user.into().mention().to_string());
        self
    }
}

impl fmt::Display for MessageBuilder {
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
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A trait with additional functionality over the [`MessageBuilder`] for
/// creating content with additional functionality available only in embeds.
///
/// Namely, this allows you to create named links via the non-escaping
/// [`Self::push_named_link`] method and the escaping [`Self::push_named_link_safe`] method.
///
/// # Examples
///
/// Make a named link to Rust's GitHub organization:
///
/// ```rust
/// use serenity::utils::{EmbedMessageBuilding, MessageBuilder};
///
/// let msg = MessageBuilder::new()
///     .push_named_link("Rust's GitHub", "https://github.com/rust-lang")
///     .build();
///
/// assert_eq!(msg, "[Rust's GitHub](https://github.com/rust-lang)");
/// ```
pub trait EmbedMessageBuilding {
    /// Pushes a named link to a message, intended for use in embeds.
    ///
    /// # Examples
    ///
    /// Make a simple link to Rust's homepage for use in an embed:
    ///
    /// ```rust
    /// use serenity::utils::{EmbedMessageBuilding, MessageBuilder};
    ///
    /// let mut msg = MessageBuilder::new();
    /// msg.push("Rust's website: ");
    /// msg.push_named_link("Homepage", "https://rust-lang.org");
    /// let content = msg.build();
    ///
    /// assert_eq!(content, "Rust's website: [Homepage](https://rust-lang.org)");
    /// ```
    fn push_named_link(&mut self, name: impl Into<Content>, url: impl Into<Content>) -> &mut Self;

    /// Pushes a named link intended for use in an embed, but with a normalized
    /// name to avoid escaping issues.
    ///
    /// Refer to [`Self::push_named_link`] for more information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::{EmbedMessageBuilding, MessageBuilder};
    ///
    /// let mut msg = MessageBuilder::new();
    /// msg.push("A weird website name: ");
    /// msg.push_named_link_safe("Try to ] break links (](", "https://rust-lang.org");
    /// let content = msg.build();
    ///
    /// assert_eq!(content, "A weird website name: [Try to   break links ( (](https://rust-lang.org)");
    /// ```
    fn push_named_link_safe(
        &mut self,
        name: impl Into<Content>,
        url: impl Into<Content>,
    ) -> &mut Self;
}

impl EmbedMessageBuilding for MessageBuilder {
    fn push_named_link(&mut self, name: impl Into<Content>, url: impl Into<Content>) -> &mut Self {
        let name = name.into().to_string();
        let url = url.into().to_string();

        write!(self.0, "[{}]({})", name, url).unwrap();

        self
    }

    fn push_named_link_safe(
        &mut self,
        name: impl Into<Content>,
        url: impl Into<Content>,
    ) -> &mut Self {
        self.0.push('[');
        {
            let mut c = name.into();
            c.inner = normalize(&c.inner).replace(']', " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push_str("](");
        {
            let mut c = url.into();
            c.inner = normalize(&c.inner).replace(')', " ");
            self.0.push_str(&c.to_string());
        }
        self.0.push(')');

        self
    }
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
/// use serenity::utils::Content;
/// use serenity::utils::ContentModifier::{Bold, Italic};
/// let content: Content = Bold + Italic + "text";
/// ```
#[non_exhaustive]
pub enum ContentModifier {
    Italic,
    Bold,
    Strikethrough,
    Code,
    Underline,
    Spoiler,
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
    pub spoiler: bool,
}

impl<T: ToString> Add<T> for Content {
    type Output = Content;

    fn add(mut self, rhs: T) -> Content {
        self.inner += &rhs.to_string();

        self
    }
}

impl<T: ToString> Add<T> for ContentModifier {
    type Output = Content;

    fn add(self, rhs: T) -> Content {
        let mut nc = self.to_content();
        nc.inner += &rhs.to_string();

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
            ContentModifier::Spoiler => {
                self.spoiler = true;
            },
        }
    }

    #[allow(clippy::inherent_to_string)]
    #[must_use]
    pub fn to_string(&self) -> String {
        trait UnwrapWith {
            fn unwrap_with(&self, n: usize) -> usize;
        }

        impl UnwrapWith for bool {
            fn unwrap_with(&self, n: usize) -> usize {
                if *self {
                    n
                } else {
                    0
                }
            }
        }

        let capacity = self.inner.len()
            + self.spoiler.unwrap_with(4)
            + self.bold.unwrap_with(4)
            + self.italic.unwrap_with(2)
            + self.strikethrough.unwrap_with(4)
            + self.underline.unwrap_with(4)
            + self.code.unwrap_with(2);

        let mut new_str = String::with_capacity(capacity);

        if self.spoiler {
            new_str.push_str("||");
        }

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

        if self.spoiler {
            new_str.push_str("||");
        }

        new_str
    }
}

impl<T: fmt::Display> From<T> for Content {
    fn from(t: T) -> Content {
        Content {
            italic: false,
            bold: false,
            strikethrough: false,
            inner: t.to_string(),
            code: false,
            underline: false,
            spoiler: false,
        }
    }
}

fn normalize(text: &str) -> String {
    // Remove invite links and popular scam websites, mostly to prevent the
    // current user from triggering various ad detectors and prevent embeds.
    text.replace("discord.gg", "discord\u{2024}gg")
        .replace("discord.me", "discord\u{2024}me")
        .replace("discordlist.net", "discordlist\u{2024}net")
        .replace("discordservers.com", "discordservers\u{2024}com")
        .replace("discord.com/invite", "discord\u{2024}com/invite")
        .replace("discordapp.com/invite", "discordapp\u{2024}com/invite")
        // Remove right-to-left override and other similar annoying symbols
        .replace([
            '\u{202E}', // RTL Override
            '\u{200F}', // RTL Mark
            '\u{202B}', // RTL Embedding
            '\u{200B}', // Zero-width space
            '\u{200D}', // Zero-width joiner
            '\u{200C}', // Zero-width non-joiner
        ], " ")
        // Remove everyone and here mentions. Has to be put after ZWS replacement
        // because it utilises it itself.
        .replace("@everyone", "@\u{200B}everyone")
        .replace("@here", "@\u{200B}here")
}

#[cfg(test)]
mod test {
    use super::ContentModifier::{Bold, Code, Italic, Spoiler};
    use super::MessageBuilder;
    use crate::model::prelude::*;

    macro_rules! gen {
        ($($fn:ident => [$($text:expr => $expected:expr),+]),+) => ({
            $(
                $(
                    assert_eq!(MessageBuilder::new().$fn($text).0, $expected);
                )+
            )+
        });
    }

    #[test]
    fn code_blocks() {
        let content = MessageBuilder::new().push_codeblock("test", Some("rb")).build();
        assert_eq!(content, "```rb\ntest\n```");
    }

    #[test]
    fn safe_content() {
        let content = MessageBuilder::new().push_safe("@everyone discord.gg/discord-api").build();
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
                available: true,
                id: EmojiId(32),
                name: "Rohrkatze".to_string(),
                managed: false,
                require_colons: true,
                roles: vec![],
                user: None,
            })
            .build();
        let content_mentions =
            MessageBuilder::new().channel(1).mention(&UserId(2)).role(3).user(4).build();
        assert_eq!(content_mentions, "<#1><@2><@&3><@4>");
        assert_eq!(content_emoji, "<:Rohrkatze:32>");
    }

    #[test]
    fn content() {
        let content = Bold + Italic + Code + "Fun!";

        assert_eq!(content.to_string(), "***`Fun!`***");

        let content = Spoiler + Bold + "Divert your eyes elsewhere";

        assert_eq!(content.to_string(), "||**Divert your eyes elsewhere**||");
    }

    #[test]
    fn init() {
        assert_eq!(MessageBuilder::new().0, "");
        assert_eq!(MessageBuilder::default().0, "");
    }

    #[test]
    fn message_content() {
        let message_content = MessageBuilder::new().push(Bold + Italic + Code + "Fun!").build();

        assert_eq!(message_content, "***`Fun!`***");
    }

    #[test]
    fn message_content_safe() {
        let message_content = MessageBuilder::new().push_safe(Bold + Italic + "test**test").build();

        assert_eq!(message_content, "***test\\*\\*test***");
    }

    #[test]
    fn push() {
        assert_eq!(MessageBuilder::new().push('a').0, "a");
        assert!(MessageBuilder::new().push("").0.is_empty());
    }

    #[test]
    fn push_codeblock() {
        let content = &MessageBuilder::new().push_codeblock("foo", None).0.clone();
        assert_eq!(content, "```\nfoo\n```");

        let content = &MessageBuilder::new().push_codeblock("fn main() { }", Some("rs")).0.clone();
        assert_eq!(content, "```rs\nfn main() { }\n```");
    }

    #[test]
    fn push_codeblock_safe() {
        assert_eq!(
            MessageBuilder::new().push_codeblock_safe("foo", Some("rs")).0,
            "```rs\nfoo\n```",
        );
        assert_eq!(MessageBuilder::new().push_codeblock_safe("", None).0, "```\n\n```",);
        assert_eq!(MessageBuilder::new().push_codeblock_safe("1 * 2", None).0, "```\n1 * 2\n```",);
        assert_eq!(
            MessageBuilder::new().push_codeblock_safe("`1 * 3`", None).0,
            "```\n`1 * 3`\n```",
        );
        assert_eq!(MessageBuilder::new().push_codeblock_safe("```.```", None).0, "```\n . \n```",);
    }

    #[test]
    fn push_safe() {
        gen! {
            push_safe => [
                "" => "",
                "foo" => "foo",
                "1 * 2" => "1 \\* 2"
            ],
            push_bold_safe => [
                "" => "****",
                "foo" => "**foo**",
                "*foo*" => "***foo***",
                "f*o**o" => "**f*o o**"
            ],
            push_italic_safe => [
                "" => "__",
                "foo" => "_foo_",
                "f_o_o" => "_f o o_"
            ],
            push_mono_safe => [
                "" => "``",
                "foo" => "`foo`",
                "asterisk *" => "`asterisk *`",
                "`ticks`" => "`'ticks'`"
            ],
            push_strike_safe => [
                "" => "~~~~",
                "foo" => "~~foo~~",
                "foo ~" => "~~foo ~~~",
                "~~foo" => "~~ foo~~",
                "~~fo~~o~~" => "~~ fo o ~~"
            ],
            push_underline_safe => [
                "" => "____",
                "foo" => "__foo__",
                "foo _" => "__foo ___",
                "__foo__ bar" => "__ foo  bar__"
            ],
            push_spoiler_safe => [
                "" => "||||",
                "foo" => "||foo||",
                "foo |" => "||foo |||",
                "||foo|| bar" =>"|| foo  bar||"
            ],
            push_line_safe => [
                "" => "\n",
                "foo" => "foo\n",
                "1 * 2" => "1 \\* 2\n"
            ],
            push_mono_line_safe => [
                "" => "``\n",
                "a ` b `" => "`a ' b '`\n"
            ],
            push_italic_line_safe => [
                "" => "__\n",
                "a * c" => "_a * c_\n"
            ],
            push_bold_line_safe => [
                "" => "****\n",
                "a ** d" => "**a   d**\n"
            ],
            push_underline_line_safe => [
                "" => "____\n",
                "a __ e" => "__a   e__\n"
            ],
            push_strike_line_safe => [
                "" => "~~~~\n",
                "a ~~ f" => "~~a   f~~\n"
            ],
            push_spoiler_line_safe => [
                "" => "||||\n",
                "a || f" => "||a   f||\n"
            ]
        };
    }

    #[test]
    fn push_unsafe() {
        gen! {
            push_bold => [
                "a" => "**a**",
                "" => "****",
                '*' => "*****",
                "**" => "******"
            ],
            push_bold_line => [
                "" => "****\n",
                "foo" => "**foo**\n"
            ],
            push_italic => [
                "a" => "_a_",
                "" => "__",
                "_" => "___",
                "__" => "____"
            ],
            push_italic_line => [
                "" => "__\n",
                "foo" => "_foo_\n",
                "_?" => "__?_\n"
            ],
            push_line => [
                "" => "\n",
                "foo" => "foo\n",
                "\n\n" => "\n\n\n",
                "\nfoo\n" => "\nfoo\n\n"
            ],
            push_mono => [
                "a" => "`a`",
                "" => "``",
                "`" => "```",
                "``" => "````"
            ],
            push_mono_line => [
                "" => "``\n",
                "foo" => "`foo`\n",
                "\n" => "`\n`\n",
                "`\n`\n" => "``\n`\n`\n"
            ],
            push_strike => [
                "a" => "~~a~~",
                "" => "~~~~",
                "~" => "~~~~~",
                "~~" => "~~~~~~"
            ],
            push_strike_line => [
                "" => "~~~~\n",
                "foo" => "~~foo~~\n"
            ],
            push_underline => [
                "a" => "__a__",
                "" => "____",
                "_" => "_____",
                "__" => "______"
            ],
            push_underline_line => [
                "" => "____\n",
                "foo" => "__foo__\n"
            ],
            push_spoiler => [
                "a" => "||a||",
                "" => "||||",
                "|" => "|||||",
                "||" => "||||||"
            ],
            push_spoiler_line => [
                "" => "||||\n",
                "foo" => "||foo||\n"
            ]
        };
    }

    #[test]
    fn normalize() {
        assert_eq!(super::normalize("@everyone"), "@\u{200B}everyone");
        assert_eq!(super::normalize("@here"), "@\u{200B}here");
        assert_eq!(super::normalize("discord.gg"), "discord\u{2024}gg");
        assert_eq!(super::normalize("discord.me"), "discord\u{2024}me");
        assert_eq!(super::normalize("discordlist.net"), "discordlist\u{2024}net");
        assert_eq!(super::normalize("discordservers.com"), "discordservers\u{2024}com");
        assert_eq!(super::normalize("discord.com/invite"), "discord\u{2024}com/invite");
        assert_eq!(super::normalize("\u{202E}"), " ");
        assert_eq!(super::normalize("\u{200F}"), " ");
        assert_eq!(super::normalize("\u{202B}"), " ");
        assert_eq!(super::normalize("\u{200B}"), " ");
        assert_eq!(super::normalize("\u{200D}"), " ");
        assert_eq!(super::normalize("\u{200C}"), " ");
    }
}
