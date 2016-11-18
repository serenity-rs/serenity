use std::collections::BTreeMap;
use std::default::Default;
use ::model::MessageId;

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
/// [`Client::get_messages`] for examples.
///
/// [`Client::get_messages`]: ../../client/struct.Client.html#method.get_messages
pub struct GetMessages(pub BTreeMap<String, u64>);

impl GetMessages {
    /// Indicates to retrieve the messages after a specific message, given by
    /// its Id.
    pub fn after<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("after".to_owned(), message_id.into().0);

        self
    }

    /// Indicates to retrieve the messages _around_ a specific message in either
    /// direction (before+after) the given message.
    pub fn around<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("around".to_owned(), message_id.into().0);

        self
    }

    /// Indicates to retrieve the messages before a specific message, given by
    /// its Id.
    pub fn before<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("before".to_owned(), message_id.into().0);

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
        self.0.insert("limit".to_owned(), if limit > 100 {
            100
        } else {
            limit
        });

        self
    }

    /// This is a function that is here for completeness. You do not need to
    /// call this - except to clear previous calls to `after`, `around`, and
    /// `before` - as it is the default value.
    pub fn most_recent(self) -> Self {
        self
    }
}

impl Default for GetMessages {
    /// Creates a builder with no default values.
    fn default() -> GetMessages {
        GetMessages(BTreeMap::default())
    }
}
