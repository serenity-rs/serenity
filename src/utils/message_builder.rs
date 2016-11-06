use std::default::Default;
use std::fmt;
use ::model::{ChannelId, Emoji, Mentionable, RoleId, UserId};

/// The Message Builder is an ergonomic utility to easily build a message,
/// by adding text and mentioning mentionable structs.
///
/// The finalized value can be accessed via `.build()` or the inner value.
///
/// # Examples
///
/// Build a message, mentioning a user and an emoji:
///
/// ```rust,ignore
/// use serenity::utils::MessageBuilder;
///
/// let content = MessageBuilder::new()
///     .push("You sent a message, ")
///     .mention(user)
///     .push("! ");
///     .mention(emoji)
///     .build();
/// ```
pub struct MessageBuilder(pub String);

impl MessageBuilder {
    /// Creates a new, empty-content builder.
    pub fn new() -> MessageBuilder {
        MessageBuilder::default()
    }

    /// Pulls the inner value out of the builder. This is equivilant to simply
    /// retrieving the value.
    pub fn build(self) -> String {
        self.0
    }

    /// Mentions the channel in the built message.
    pub fn channel<C: Into<ChannelId>>(mut self, channel: C) -> Self {
        self.0.push_str(&format!("{}", channel.into()));

        self
    }

    /// Uses and displays the given emoji in the built message.
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
    pub fn push(mut self, content: &str) -> Self {
        self.0.push_str(content);

        self
    }


    /// Mentions the role in the built message.
    pub fn role<R: Into<RoleId>>(mut self, role: R) -> Self {
        self.0.push_str(&format!("{}", role.into()));

        self
    }

    /// Mentions the user in the built message.
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
