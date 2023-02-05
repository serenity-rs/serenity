#[cfg(feature = "http")]
use crate::http::{Http, MessagePagination};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Builds a request to the API to retrieve messages.
///
/// This accepts 2 types of parameters. The first type filters messages based on Id, and is set by
/// one of the following:
///
/// - [`Self::after`]
/// - [`Self::around`]
/// - [`Self::before`]
///
/// These are mutually exclusive, and override each other if called sequentially. If one is not
/// specified, messages are simply sorted by most recent.
///
/// The other parameter specifies the number of messages fo retrieve. This is _optional_, and
/// defaults to 50 if not specified.
///
/// See [`GuildChannel::messages`] for more examples.
///
/// # Examples
///
/// Creating a [`GetMessages`] builder to retrieve the first 25 messages after the message with an
/// Id of `158339864557912064`:
///
/// ```rust,no_run
/// # use serenity::http::Http;
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http: Http = unimplemented!();
/// use serenity::builder::GetMessages;
/// use serenity::model::id::{ChannelId, MessageId};
///
/// // you can then pass it into a function which retrieves messages:
/// let channel_id = ChannelId::new(81384788765712384);
///
/// let builder = GetMessages::new().after(MessageId::new(158339864557912064)).limit(25);
/// let _messages = channel_id.messages(&http, builder).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default)]
#[must_use]
pub struct GetMessages {
    search_filter: Option<SearchFilter>,
    limit: Option<u8>,
}

impl GetMessages {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets messages from the channel.
    ///
    /// **Note**: If the user does not have the [Read Message History] permission, returns an empty
    /// [`Vec`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: impl AsRef<Http>,
        channel_id: ChannelId,
    ) -> Result<Vec<Message>> {
        http.as_ref().get_messages(channel_id, self.search_filter.map(Into::into), self.limit).await
    }

    /// Indicates to retrieve the messages after a specific message, given its Id.
    pub fn after(mut self, message_id: impl Into<MessageId>) -> Self {
        self.search_filter = Some(SearchFilter::After(message_id.into()));
        self
    }

    /// Indicates to retrieve the messages _around_ a specific message, in other words in either
    /// direction from the message in time.
    pub fn around(mut self, message_id: impl Into<MessageId>) -> Self {
        self.search_filter = Some(SearchFilter::Around(message_id.into()));
        self
    }

    /// Indicates to retrieve the messages before a specific message, given its Id.
    pub fn before(mut self, message_id: impl Into<MessageId>) -> Self {
        self.search_filter = Some(SearchFilter::Before(message_id.into()));
        self
    }

    /// The maximum number of messages to retrieve for the query.
    ///
    /// If this is not specified, a default value of 50 is used.
    ///
    /// **Note**: This field is capped to 100 messages due to a Discord limitation. If an amount
    /// larger than 100 is supplied, it will be truncated.
    pub fn limit(mut self, limit: u8) -> Self {
        self.limit = Some(limit.min(100));
        self
    }
}

#[derive(Clone, Copy, Debug)]
enum SearchFilter {
    After(MessageId),
    Around(MessageId),
    Before(MessageId),
}

#[cfg(feature = "http")]
impl From<SearchFilter> for MessagePagination {
    fn from(filter: SearchFilter) -> Self {
        match filter {
            SearchFilter::After(id) => MessagePagination::After(id),
            SearchFilter::Around(id) => MessagePagination::Around(id),
            SearchFilter::Before(id) => MessagePagination::Before(id),
        }
    }
}
