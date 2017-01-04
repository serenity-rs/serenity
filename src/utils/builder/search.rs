use std::collections::BTreeMap;
use ::model::{MessageId, UserId};

/// An indicator of the type of sorting mode to use when searching for
/// [`Message`]s via the [`Search`] builder.
///
/// [`Message`]: ../../model/struct.Message.html
/// [`Search`]: struct.Search.html
pub enum SortingMode {
    /// Search by messages' relevance to parameters.
    Relevance,
    /// Search by messages' timestamp, where results will match according to
    /// parameters. This is used in conjunction in the [`Search::sort_order`]
    /// method, and is used in conjunction with [`SortingOrder`].
    ///
    /// [`Search::sort_order`]: struct.Search.html#method.sort_order
    /// [`SortingOrder`]: enum.SortingOrder.html
    Timestamp,
}

impl SortingMode {
    /// Retrieves the name of the sorting mode. This is equivalent to a
    /// lowercase string version of each variant.
    pub fn name(&self) -> &str {
        match *self {
            SortingMode::Relevance => "relevance",
            SortingMode::Timestamp => "timestamp",
        }
    }
}

/// An indicator of how to sort results when searching for [`Message`]s via the
/// [`Search`] builder.
///
/// [`Message`]: ../../model/struct.Message.html
/// [`Search`]: struct.Search.html
pub enum SortingOrder {
    /// Search message results in ascending order.
    ///
    /// In the case of [`SortingMode::Relevance`], this will search from the
    /// least relevant to the most relevant.
    ///
    /// In the case of [`SortingMode::Timestamp`], this will indicate to search
    /// from the least recent to the most recent.
    ///
    /// [`SortingMode::Relevance`]: enum.SortingMode.html#variant.Relevance
    /// [`SortingMode::Timestamp`]: enum.SortingMode.html#variant.Timestamp
    Ascending,
    /// Search message results in descending order.
    ///
    /// In the case of [`SortingMode::Relevance`], this will search from the
    /// most relevant to least relevant.
    ///
    /// In the case of [`SortingMode::Timestamp`], this will search from the
    /// most recent to least recent.
    ///
    /// [`SortingMode::Relevance`]: enum.SortingMode.html#variant.Relevance
    /// [`SortingMode::Timestamp`]: enum.SortingMode.html#variant.Timestamp
    Descending,
}

impl SortingOrder {
    /// Retrieves the name of the sorting order. This is equivalent to a
    /// lowercase string version of each variant.
    pub fn name(&self) -> &str {
        match *self {
            SortingOrder::Ascending => "asc",
            SortingOrder::Descending => "desc",
        }
    }
}

/// A builder used to query a [`Channel`] or [`Guild`] for its [`Message`]s,
/// specifying certain parameters to narrow down the returned messages.
///
/// Many methods are provided to narrow down the results, such as [`sort_by`] -
/// which is used with the [`SortingMode`] enum to sort the results - or
/// [`limit`], which can be used in conjunction with [`offset`] to paginate
/// results.
///
/// # Examples
///
/// Provided are multiple in-depth examples for searching through different
/// means. Also see [example 08] for a fully runnable bot.
///
/// ### Searching a Channel
///
/// Search for messages via [`Context::search_channel`] with the content
/// `"rust"`, which have no embed, no attachment, searching by relevance in
/// ascending order, and limiting to 5 results:
///
/// ```rust,ignore
/// // assuming you are in a context
///
/// let res = context.search_channel(message.channel_id, |s| s
///     .content("rust")
///     .has_embed(false)
///     .has_attachment(false)
///     .limit(5)
///     .sort_by(SortingMode::Relevance)
///     .sort_order(SortingOrder::Ascending));
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
/// use serenity::utils::builder::{SortingMode, SortingOrder};
/// use std::env;
///
/// let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN").unwrap());
///
/// client.with_framework(|f| f
///     .configure(|c| c.prefix("~").on_mention(true))
///     .on("search", search));
///
/// command!(search(context, message, args) {
///     let query = args.join(" ");
///
///     if query.is_empty() {
///         let _ = context.say("You must provide a query");
///
///         return Ok(());
///     }
///
///     let guild = message.guild().unwrap();
///
///     let channel_ids = guild
///         .channels
///         .values()
///         .filter(|c| c.name.starts_with("search-"))
///         .map(|c| c.id)
///         .collect();
///
///     let search = context.search_guild(guild.id, channel_ids, |s| s
///         .content(&query)
///         .context_size(0)
///         .has_attachment(true)
///         .has_embed(true)
///         .max_id(message.id.0 - 1)
///         .sort_by(SortingMode::Timestamp)
///         .sort_order(SortingOrder::Descending));
///
///     let mut messages = match search {
///         Ok(messages) => messages,
///         Err(why) => {
///             println!("Error performing search '{}': {:?}", query, why);
///
///             let _ = context.say("Error occurred while searching");
///
///             return Ok(());
///         },
///     };
///
///     let _ = context.send_message(message.channel_id, |m| m
///         .content(&format!("Found {} total results", messages.total))
///         .embed(|mut e| {
///             for (i, messages) in messages.results.iter_mut().enumerate() {
///                 let mut message = match messages.get_mut(i) {
///                     Some(message) => message,
///                     None => break,
///                 };
///
///                 message.content.truncate(1000);
///
///                 e = e.field(|f| f
///                     .name(&format!("Result {}", i))
///                     .value(&message.content));
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
/// [`SortingMode`]: enum.SortingMode.html
/// [`limit`]: #method.limit
/// [`offset`]: #method.offset
/// [`sort_by`]: #method.sort_by
/// [example 08]: https://github.com/zeyla/serenity/tree/master/examples/08_search
pub struct Search<'a>(pub BTreeMap<&'a str, String>);

impl<'a> Search<'a> {
    /// Sets the list of attachment extensions to search by.
    ///
    /// When providing a vector of extensions, do _not_ include the period (`.`)
    /// character as part of the search.
    ///
    /// This is sent to Discord as a comma-separated value list of extension
    /// names.
    pub fn attachment_extensions(mut self, attachment_extensions: &[&str]) -> Self {
        let list = attachment_extensions.join(" ");

        self.0.insert("attachment_extensions", list);

        self
    }

    /// Sets the filename of the attachments to search for.
    pub fn attachment_filename(mut self, attachment_filename: &str) -> Self {
        self.0.insert("attachment_filename", attachment_filename.to_owned());

        self
    }

    /// Sets the Id of the author of [`Message`]s to search for. This excludes
    /// all messages by other [`User`]s.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    /// [`User`]: ../../model/struct.User.html
    pub fn author_id<U: Into<UserId>>(mut self, author_id: U) -> Self {
        self.0.insert("author_id", author_id.into().0.to_string());

        self
    }

    /// Sets the content of the [`Message`] to search for. This is a fuzzy
    /// search, and can partially match the given query content.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn content(mut self, content: &str) -> Self {
        self.0.insert("content", content.to_owned());

        self
    }

    /// Sets the amount of "context" [`Message`]s to provide, at maximum. This
    /// is the number of messages to provide around each side
    /// (ascending+descending) of the "hit" (aka found) message.
    ///
    /// The default value is `2`. The minimum value is `0`. The maximum value is
    /// `2`.
    pub fn context_size(mut self, context_size: u8) -> Self {
        self.0.insert("context_size", context_size.to_string());

        self
    }

    /// Sets the embed providers to search by.
    ///
    /// This is a list of the providers' names.
    ///
    /// This is sent to Discord as a comma-separated value list of provider
    /// names.
    pub fn embed_providers(mut self, embed_providers: &[&str]) -> Self {
        self.0.insert("embed_providers", embed_providers.join(" "));

        self
    }

    /// Sets the type of [`Embed`]s to search by.
    ///
    /// An example of an [embed type][`Embed::kind`] is `"rich"`.
    ///
    /// [`Embed`]: ../../model/struct.Embed.html
    /// [`Embed::kind`]: ../../model/struct.Embed.html#structfield.kind
    pub fn embed_types(mut self, embed_types: &[&str]) -> Self {
        self.0.insert("embed_types", embed_types.join(" "));

        self
    }

    /// Sets whether to search for methods that do - or do not - have an
    /// attachment.
    ///
    /// Do not specify to search for both.
    pub fn has_attachment(mut self, has_attachment: bool) -> Self {
        self.0.insert("has_attachment", has_attachment.to_string());

        self
    }

    /// Sets whether to search for methods that do - or do not - have an embed.
    ///
    /// Do not specify to search for both.
    pub fn has_embed(mut self, has_embed: bool) -> Self {
        self.0.insert("has_embed", has_embed.to_string());

        self
    }

    /// Sets the number of messages to retrieve _at maximum_. This can be used
    /// in conjunction with [`offset`].
    ///
    /// The minimum value is `1`. The maximum value is `25`.
    ///
    /// [`offset`]: #method.offset
    pub fn limit(mut self, limit: u8) -> Self {
        self.0.insert("limit", limit.to_string());

        self
    }

    /// Set the maximum [`Message`] Id to search up to. All messages with an Id
    /// greater than the given value will be ignored.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn max_id<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("max_id", message_id.into().0.to_string());

        self
    }

    /// Set the minimum [`Message`]s Id to search down to. All messages with an
    /// Id less than the given value will be ignored.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    pub fn min_id<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("min_id", message_id.into().0.to_string());

        self
    }

    /// Set the offset of [`Message`]s to return. This can be used in
    /// conjunction with [`limit`].
    ///
    /// The minimum value is `0`. The maximum value is `5000`.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    /// [`limit`]: fn.limit.html
    pub fn offset(mut self, offset: u16) -> Self {
        self.0.insert("offset", offset.to_string());

        self
    }

    /// The sorting mode to use.
    ///
    /// Refer to [`SortingMode`] for more information.
    ///
    /// [`SortingMode`]: enum.SortingMode.html
    pub fn sort_by(mut self, sorting_mode: SortingMode) -> Self {
        self.0.insert("sort_by", sorting_mode.name().to_string());

        self
    }

    /// The order to sort results by.
    ///
    /// Refer to the documentation for [`SortingOrder`] for more information.
    ///
    /// [`SortingOrder`]: enum.SortingOrder.html
    pub fn sort_order(mut self, sorting_order: SortingOrder) -> Self {
        self.0.insert("sort_order", sorting_order.name().to_string());

        self
    }
}

impl<'a> Default for Search<'a> {
    /// Creates a new builder for searching for [`Message`]s. Refer to each
    /// method to learn what minimum and maximum values are available for each
    /// field, as well as restrictions and other useful information.
    ///
    /// The library does not provide defaults differently than what Discord
    /// itself defaults to.
    ///
    /// This list of defaults is:
    ///
    /// - [`context_size`]: 2
    /// - [`limit`]: 25
    /// - [`offset`]: 0
    /// - [`sort_by`]: [`SortingMode::Timestamp`]
    ///
    /// [`SortingMode::Timestamp`]: enum.SortingMode.html#variant.Timestamp
    /// [`context_size`]: #method.context_size
    /// [`limit`]: #method.limit
    /// [`offset`]: #method.offset
    /// [`sort_by`]: #method.sort_by
    fn default<'b>() -> Search<'b> {
        Search(BTreeMap::default())
    }
}
