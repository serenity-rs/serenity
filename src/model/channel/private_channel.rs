use std::fmt;

#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;
use crate::model::utils::single_recipient;

/// A Direct Message text channel with another user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PrivateChannel {
    /// The unique Id of the private channel.
    ///
    /// Can be used to calculate the first message's creation date.
    pub id: ChannelId,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the last time a [`Message`] was pinned.
    pub last_pin_timestamp: Option<Timestamp>,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Private`].
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The recipient to the private channel.
    #[serde(with = "single_recipient", rename = "recipients")]
    pub recipient: User,
}

#[cfg(feature = "model")]
impl PrivateChannel {
    /// Deletes the channel. This does not delete the contents of the channel, and is equivalent to
    /// closing a private channel on the client, which can be re-opened.
    #[expect(clippy::missing_errors_doc)]
    pub async fn delete(&self, http: &Http) -> Result<PrivateChannel> {
        let resp = self.id.delete(http, None).await?;
        resp.private().ok_or(Error::Model(ModelError::InvalidChannelType))
    }
}

impl fmt::Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.recipient.name)
    }
}
