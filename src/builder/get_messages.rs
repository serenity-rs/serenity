#[cfg(feature = "http")]
use std::fmt::Write;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Builds a request to the API to retrieve messages.
///
/// This accepts 2 types of parameters. The first filters messages based on Id, and is set by one
/// of the following:
///
/// - [`Self::after`]
/// - [`Self::around`]
/// - [`Self::before`]
///
/// These are mutually exclusive, and override each other if called sequentially. If one is not
/// specified, messages are simply sorted by most recent.
///
/// The other parameter specifies number of messages to retrieve. This is _optional_, and defaults
/// to 50 if not specified.
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
/// # let http = Http::new("token");
/// use serenity::model::id::{ChannelId, MessageId};
///
/// // you can then pass it into a function which retrieves messages:
/// let channel_id = ChannelId::new(81384788765712384);
///
/// let _messages = channel_id
///     .messages()
///     .after(MessageId::new(158339864557912064))
///     .limit(25)
///     .execute(&http)
///     .await?;
/// #     Ok(())
/// # }
/// ```
///
/// [`GuildChannel::messages`]: crate::model::channel::GuildChannel::messages
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct GetMessages {
    #[cfg(feature = "http")]
    id: ChannelId,
    search_filter: Option<SearchFilter>,
    limit: Option<u8>,
}

impl GetMessages {
    pub fn new(#[cfg(feature = "http")] id: ChannelId) -> Self {
        Self {
            #[cfg(feature = "http")]
            id,
            search_filter: None,
            limit: None,
        }
    }

    /// Indicates to retrieve the messages after a specific message, given its Id.
    #[inline]
    pub fn after<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self._after(message_id.into());
        self
    }

    fn _after(&mut self, message_id: MessageId) {
        self.search_filter = Some(SearchFilter::After(message_id));
    }

    /// Indicates to retrieve the messages _around_ a specific message, in other words in either
    /// direction from the message in time.
    #[inline]
    pub fn around<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self._around(message_id.into());
        self
    }

    fn _around(&mut self, message_id: MessageId) {
        self.search_filter = Some(SearchFilter::Around(message_id));
    }

    /// Indicates to retrieve the messages before a specific message, given its Id.
    #[inline]
    pub fn before<M: Into<MessageId>>(mut self, message_id: M) -> Self {
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
    /// **Note**: This field is capped to 100 messages due to a Discord limitation. If an amount
    /// larger than 100 is supplied, it will be truncated.
    pub fn limit(mut self, limit: u8) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Gets messages from the channel.
    ///
    /// **Note**: If the user does not have the [Read Message History] permission, returns an empty
    /// [`Vec`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission to view the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        let mut query = "?".to_string();
        if let Some(limit) = self.limit {
            write!(query, "limit={}", limit)?;
        }

        if let Some(filter) = self.search_filter {
            match filter {
                SearchFilter::After(after) => write!(query, "&after={}", after)?,
                SearchFilter::Around(around) => write!(query, "&around={}", around)?,
                SearchFilter::Before(before) => write!(query, "&before={}", before)?,
            }
        }

        http.as_ref().get_messages(self.id.into(), &query).await
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SearchFilter {
    After(MessageId),
    Around(MessageId),
    Before(MessageId),
}
