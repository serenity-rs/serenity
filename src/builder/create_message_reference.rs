use crate::model::prelude::*;

/// Reference data sent with crossposted messages.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#message-reference-object-message-reference-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CreateMessageReference {
    /// The type of Message Reference.
    #[serde(rename = "type", default = "MessageReferenceKind::default")]
    pub kind: MessageReferenceKind,
    /// The ID of the originating messages's channel.
    pub channel_id: ChannelId,
    /// The ID of the originating message.
    pub message_id: Option<MessageId>,
    /// The ID of the originating messages's guild.
    pub guild_id: Option<GuildId>,
    /// When sending, whether to error if the referenced message doesn't exist instead of sending
    /// as a normal(non-reply) message, default true.
    pub fail_if_not_exists: Option<bool>,
}

impl CreateMessageReference {
    #[must_use]
    pub fn new(kind: MessageReferenceKind, message_id: MessageId, channel_id: ChannelId) -> Self {
        Self {
            kind,
            channel_id,
            message_id: Some(message_id),
            guild_id: None,
            fail_if_not_exists: None,
        }
    }

    #[must_use]
    pub fn guild(mut self, guild: GuildId) -> Self {
        self.guild_id = Some(guild);
        self
    }

    #[must_use]
    pub fn fail_if_not_exists(mut self, fail_if_not_exists: bool) -> Self {
        self.fail_if_not_exists = Some(fail_if_not_exists);
        self
    }
}

impl From<MessageReference> for CreateMessageReference {
    fn from(value: MessageReference) -> Self {
        Self {
            kind: value.kind,
            channel_id: value.channel_id,
            message_id: value.message_id,
            guild_id: value.guild_id,
            fail_if_not_exists: value.fail_if_not_exists,
        }
    }
}

impl From<&Message> for CreateMessageReference {
    fn from(value: &Message) -> Self {
        Self {
            kind: MessageReferenceKind::default(),
            channel_id: value.channel_id,
            message_id: Some(value.id),
            guild_id: value.guild_id,
            fail_if_not_exists: None,
        }
    }
}
