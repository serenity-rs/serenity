use crate::model::id::MessageId;

/// Builds a request to the API to retrieve messages.
///
/// This can have 2 different sets of parameters. The first set is around where
/// to get the messages:
///
/// - [`Self::after`]
/// - [`Self::around`]
/// - [`Self::before`]
///
/// These can not be mixed, and the first in the list alphabetically will be
/// used. If one is not specified, `most_recent` will be used.
///
/// The fourth parameter is to specify the number of messages to retrieve. This
/// does not _need_ to be called and defaults to a value of 50.
///
/// This should be used only for retrieving messages; see
/// [`GuildChannel::messages`] for examples.
///
/// # Examples
///
/// Creating a [`GetMessages`] builder to retrieve the first 25 messages after the
/// message with an Id of `158339864557912064`:
///
/// ```rust,no_run
/// # use serenity::http::Http;
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http = Http::new("token");
/// use serenity::model::id::{ChannelId, MessageId};
///
/// // you can then pass it into a function which retrieves messages:
/// let channel_id = ChannelId::new(81384788765712384);
///
/// let _messages = channel_id
///     .messages(&http, |retriever| retriever.after(MessageId::new(158339864557912064)).limit(25))
///     .await?;
/// #     Ok(())
/// # }
/// ```
///
/// [`GuildChannel::messages`]: crate::model::channel::GuildChannel::messages
#[derive(Clone, Copy, Debug, Default)]
pub struct GetMessages {
    pub search_filter: Option<SearchFilter>,
    pub limit: Option<u8>,
}

impl GetMessages {
    /// Indicates to retrieve the messages after a specific message, given by
    /// its Id.
    #[inline]
    pub fn after<M: Into<MessageId>>(&mut self, message_id: M) -> &mut Self {
        self._after(message_id.into());
        self
    }

    fn _after(&mut self, message_id: MessageId) {
        self.search_filter = Some(SearchFilter::After(message_id));
    }

    /// Indicates to retrieve the messages _around_ a specific message in either
    /// direction (before+after) the given message.
    #[inline]
    pub fn around<M: Into<MessageId>>(&mut self, message_id: M) -> &mut Self {
        self._around(message_id.into());
        self
    }

    fn _around(&mut self, message_id: MessageId) {
        self.search_filter = Some(SearchFilter::Around(message_id));
    }

    /// Indicates to retrieve the messages before a specific message, given by
    /// its Id.
    #[inline]
    pub fn before<M: Into<MessageId>>(&mut self, message_id: M) -> &mut Self {
        self._before(message_id.into());
        self
    }

    fn _before(&mut self, message_id: MessageId) {
        self.search_filter = Some(SearchFilter::Before(message_id));
    }

    /// The maximum number of messages to retrieve for the query.
    ///
    /// If this is not specified, a default value of 50 is used.
    ///
    /// **Note**: This field is capped to 100 messages due to a Discord
    /// limitation. If an amount larger than 100 is supplied, it will be
    /// reduced.
    pub fn limit(&mut self, limit: u8) -> &mut Self {
        self.limit = Some(limit.min(100));
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SearchFilter {
    After(MessageId),
    Around(MessageId),
    Before(MessageId),
}
