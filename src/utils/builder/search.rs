use serde_json::Value;
use std::collections::BTreeMap;
use ::model::{MessageId, UserId};

/// An indicator for filtering [`Message`]s that have a certain item or quality.
///
/// Used with [`Search::has`].
///
/// [`Message`]: ../../model/struct.Message.html
/// [`Search::has`]: struct.Search.html#method.has
pub enum Has {
    /// Find messages that have an [`Embed`].
    ///
    /// [`Embed`]: ../../model/struct.Embed.html
    Embed,
    /// Find messages that have an [`Attachment`].
    ///
    /// [`Attachment`]: ../../model/struct.Attachment.html
    File,
    /// Find messages that have an embed with an image.
    Image,
    /// Find messages with a link of any kind.
    Link,
    /// Find messages that have an [`Embed`] with a sound [`provider`].
    ///
    /// [`Embed`]: ../../model/struct.Embed.html
    /// [`provider`]: ../../model/struct.Embed.html#structfield.provider
    Sound,
    /// Find messages that have an [`Embed`] with a `video`
    /// [type][`Embed::kind`].
    ///
    /// [`Embed`]: ../../model/struct.Embed.html
    /// [`Embed::kind`]: ../../model/struct.Embed.html#structfield.kind
    Video,
}

impl Has {
    /// Returns the "name" of the variant.
    #[doc(hidden)]
    pub fn name(&self) -> &str {
        use self::Has::*;

        match *self {
            Embed => "embed",
            File => "file",
            Image => "image",
            Link => "link",
            Sound => "sound",
            Video => "video",
        }
    }
}

/// A builder used to query a [`Channel`] or [`Guild`] for its [`Message`]s,
/// specifying certain parameters to narrow down the returned messages.
///
/// Many methods are provided to narrow down the results, such as [`limit`],
/// which can be used in conjunction with [`offset`] to paginate results.
///
/// # Examples
///
/// Provided are multiple in-depth examples for searching through different
/// means. Also see [example 08] for a fully runnable selfbot.
///
/// ### Searching a Channel
///
/// Search for messages via [`Channel::search`] with the content `"rust"`, which
/// have an embed, have a video, and limiting to 5 results:
///
/// ```rust,ignore
/// use serenity::utils::builder::Has;
///
/// // assuming a `channel` has been bound
///
/// let search = channel.search(|s| s
///     .content("rust")
///     .has(vec![Has::Embed, Has::Video])
///     .limit(5));
/// ```
///
/// ### Searching a Guild's Channels
///
/// Search for messages with a query provided by a user, which have an
/// embed, have no attachment, searching by timestamp in descending order,
/// limiting to 2 results, and only searching channels that have a name
/// prefixed with `"search-"`:
///
/// ```rust,ignore
/// use serenity::client::{Client, Context};
/// use serenity::model::Message;
/// use std::env;
///
/// let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN").unwrap());
///
/// client.with_framework(|f| f
///     .configure(|c| c.prefix("~").on_mention(true))
///     .on("search", search));
///
/// command!(search(ctx, msg, args) {
///     let query = args.join(" ");
///
///     if query.is_empty() {
///         let _ = msg.channel_id.say("You must provide a query");
///
///         return Ok(());
///     }
///
///     let guild = msg.guild().unwrap();
///
///     let channel_ids = guild
///         .channels
///         .values()
///         .filter(|c| c.name.starts_with("search-"))
///         .map(|c| c.id)
///         .collect();
///
///     let search = guild.search(guild.id, channel_ids, |s| s
///         .content(&query)
///         .context_size(0)
///         .has_attachment(true)
///         .has_embed(true)
///         .max_id(msg.id.0 - 1))
///         .unwrap();
///
///     let _ = msg.channel_id.send_message(|m| m
///         .content(&format!("Found {} total results", messages.total))
///         .embed(|mut e| {
///             for (i, messages) in messages.results.iter_mut().enumerate() {
///                 let mut found = match messages.get_mut(i) {
///                     Some(found) => found,
///                     None => break,
///                 };
///
///                 found.content.truncate(1000);
///
///                 e = e.field(|f| f
///                     .name(&format!("Result {}", i))
///                     .value(&found.content));
///              }
///
///              e
///         }));
/// });
/// ```
///
/// [`Channel`]: ../../model/enum.Channel.html
/// [`Context::search_channel`]: ../../client/struct.Context.html#method.search_channel
/// [`Guild`]: ../../model/struct.Guild.html
/// [`Message`]: ../../model/struct.Message.html
/// [`limit`]: #method.limit
/// [`offset`]: #method.offset
/// [example 08]: https://github.com/zeyla/serenity/tree/master/examples/08_search
pub struct Search(pub BTreeMap<&'static str, Value>);

impl Search {
    /// Filters [`Message`]s by the the Id of the author.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn author_id<U: Into<UserId>>(mut self, author_id: U) -> Self {
        self.0.insert("author_id", Value::U64(author_id.into().0));

        self
    }

    /// Filtes [`Message`] by content. This is a fuzzy search, and can partially
    /// match the given query content.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn content<S: Into<String>>(mut self, content: S) -> Self {
        self.0.insert("content", Value::String(content.into()));

        self
    }

    /// Sets the amount of "contextual" [`Message`]s to provide, at maximum.
    /// This is the number of messages to provide around each side
    /// (ascending+descending) of the "hit" (aka found) message.
    ///
    /// The number of returned contextual messages can be lower if there are
    /// fewer messages in the order.
    ///
    /// The default value is `2`. The minimum value is `0`. The maximum value is
    /// `2`.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn context_size(mut self, mut context_size: u8) -> Self {
        if context_size > 2 {
            context_size = 2;
        }

        self.0.insert("context_size", Value::U64(context_size as u64));

        self
    }

    /// Filter [`Message`]s by whether they have a certain item.
    ///
    /// You can pass either one or more [`Has`] variants.
    ///
    /// # Examples
    ///
    /// Passing a single variant:
    ///
    /// ```rust,no_run
    /// use serenity::model::ChannelId;
    /// use serenity::utils::builder::Has;
    ///
    /// let _ = ChannelId(7).search(|s| s.has(vec![Has::Embed]));
    /// ```
    ///
    /// Passing multiple:
    ///
    /// ```rust,no_run
    /// use serenity::model::ChannelId;
    /// use serenity::utils::builder::Has;
    ///
    /// let _ = ChannelId(7).search(|s| s.has(vec![Has::Embed, Has::Sound]));
    /// ```
    ///
    /// [`Has`]: enum.Has.html
    /// [`Message`]: ../../model/struct.Message.html
    pub fn has(mut self, has: Vec<Has>) -> Self {
        let names = has.into_iter().map(|h| Value::String(h.name().to_owned())).collect();
        self.0.insert("has", Value::Array(names));

        self
    }

    /// Sets the number of messages to retrieve _at maximum_. This can be used
    /// in conjunction with [`offset`].
    ///
    /// The minimum value is `1`. The maximum value is `25`.
    ///
    /// [`offset`]: #method.offset
    pub fn limit(mut self, limit: u8) -> Self {
        self.0.insert("limit", Value::U64(limit as u64));

        self
    }

    /// Set the maximum [`Message`] Id to search up to. All messages with an Id
    /// greater than the given value will be ignored.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn max_id<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("max_id", Value::U64(message_id.into().0));

        self
    }

    /// Filter [`Message`]s by whether they mention one or more specific
    /// [`User`]s.
    ///
    /// This is an OR statement.
    ///
    /// # Examples
    ///
    /// Search for only one mention:
    ///
    /// ```rust,no_run
    /// use serenity::model::{ChannelId, UserId};
    ///
    /// let _ = ChannelId(7).search(|s| s.mentions(vec![UserId(8)]));
    /// ```
    ///
    /// Search for two mentions:
    ///
    /// ```rust,no_run
    /// use serenity::model::{ChannelId, UserId};
    ///
    /// let _ = ChannelId(7).search(|s| s.mentions(vec![UserId(8), UserId(9)]));
    /// ```
    ///
    /// [`Message`]: ../../model/struct.Message.html
    /// [`User`]: ../../model/struct.User.html
    pub fn mentions(mut self, mentions: Vec<UserId>) -> Self {
        let ids = mentions.into_iter().map(|m| Value::U64(m.0)).collect();
        self.0.insert("mentions", Value::Array(ids));

        self
    }

    /// Set the minimum [`Message`]s Id to search down to. All messages with an
    /// Id less than the given value will be ignored.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn min_id<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("min_id", Value::U64(message_id.into().0));

        self
    }

    /// Set the offset of [`Message`]s to return. This can be used in
    /// conjunction with [`limit`].
    ///
    /// The minimum value is `0`. The maximum value is `5000`.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    /// [`limit`]: #method.limit
    pub fn offset(mut self, offset: u16) -> Self {
        self.0.insert("offset", Value::U64(offset as u64));

        self
    }
}

impl Default for Search {
    /// Creates a new builder for searching for [`Message`]s. Refer to each
    /// method to learn what minimum and maximum values are available for each
    /// field, as well as restrictions and other useful information.
    ///
    /// The library does not provide defaults differently than what Discord
    /// itself defaults to.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    fn default() -> Search {
        Search(BTreeMap::default())
    }
}
