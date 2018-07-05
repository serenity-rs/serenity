use model::id::MessageId;
use utils::VecMap;

/// Builds a request for a request to the API to retrieve messages.
///
/// This can have 2 different sets of parameters. The first set is around where
/// to get the messages:
///
/// - `after`
/// - `around`
/// - `before`
/// - `most_recent`
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
/// Creating a `GetMessages` builder to retrieve the first 25 messages after the
/// message with an Id of `158339864557912064`:
///
/// ```rust,no_run
/// # use std::error::Error;
/// #
/// # fn try_main() -> Result<(), Box<Error>> {
/// use serenity::builder::GetMessages;
/// use serenity::model::id::{ChannelId, MessageId};
///
/// let retriever = GetMessages::default()
///     .after(MessageId(158339864557912064))
///     .limit(25);
///
/// // you can then pass it into a function which retrieves messages:
/// let channel_id = ChannelId(81384788765712384);
///
/// let _messages = channel_id.messages(|_| retriever)?;
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
/// ```
///
/// [`GuildChannel::messages`]: ../model/channel/struct.GuildChannel.html#method.messages
#[derive(Clone, Debug, Default)]
pub struct GetMessages(pub VecMap<&'static str, u64>);

impl GetMessages {
    /// Indicates to retrieve the messages after a specific message, given by
    /// its Id.
    #[inline]
    pub fn after<M: Into<MessageId>>(self, message_id: M) -> Self {
        self._after(message_id.into())
    }

    fn _after(mut self, message_id: MessageId) -> Self {
        self.0.insert("after", message_id.0);

        self
    }

    /// Indicates to retrieve the messages _around_ a specific message in either
    /// direction (before+after) the given message.
    #[inline]
    pub fn around<M: Into<MessageId>>(self, message_id: M) -> Self {
        self._around(message_id.into())
    }

    fn _around(mut self, message_id: MessageId) -> Self {
        self.0.insert("around", message_id.0);

        self
    }

    /// Indicates to retrieve the messages before a specific message, given by
    /// its Id.
    #[inline]
    pub fn before<M: Into<MessageId>>(self, message_id: M) -> Self {
        self._before(message_id.into())
    }

    fn _before(mut self, message_id: MessageId) -> Self {
        self.0.insert("before", message_id.0);

        self
    }

    /// The maximum number of messages to retrieve for the query.
    ///
    /// If this is not specified, a default value of 50 is used.
    ///
    /// **Note**: This field is capped to 100 messages due to a Discord
    /// limitation. If an amount larger than 100 is supplied, it will be
    /// reduced.
    pub fn limit(mut self, limit: u64) -> Self {
        self.0
            .insert("limit", if limit > 100 { 100 } else { limit });

        self
    }

    /// This is a function that is here for completeness. You do not need to
    /// call this - except to clear previous calls to `after`, `around`, and
    /// `before` - as it is the default value.
    pub fn most_recent(self) -> Self { self }
}
