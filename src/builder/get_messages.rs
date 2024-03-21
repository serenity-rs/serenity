#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::{CacheHttp, MessagePagination};
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
/// The other parameter specifies the number of messages to retrieve. This is _optional_, and
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
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#get-channel-messages)
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

    /// Indicates to retrieve the messages after a specific message, given its Id.
    pub fn after(mut self, message_id: MessageId) -> Self {
        self.search_filter = Some(SearchFilter::After(message_id));
        self
    }

    /// Indicates to retrieve the messages _around_ a specific message, in other words in either
    /// direction from the message in time.
    pub fn around(mut self, message_id: MessageId) -> Self {
        self.search_filter = Some(SearchFilter::Around(message_id));
        self
    }

    /// Indicates to retrieve the messages before a specific message, given its Id.
    pub fn before(mut self, message_id: MessageId) -> Self {
        self.search_filter = Some(SearchFilter::Before(message_id));
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

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for GetMessages {
    type Context<'ctx> = ChannelId;
    type Built = Vec<Message>;

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
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        cache_http.http().get_messages(ctx, self.search_filter.map(Into::into), self.limit).await
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
