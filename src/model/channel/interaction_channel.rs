use nonmax::NonMaxU16;

use crate::model::prelude::*;

/// Represents the shared fields between a [`InteractionChannel`] and a [`InteractionGuildThread`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseInteractionChannel {
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The Id of the guild the channel is located in.
    #[serde(default)]
    pub guild_id: GuildId,
    /// The channel name.
    pub name: Option<FixedString>,
    /// The Id of the last message sent in the channel (or thread for forum channels).
    ///
    /// **Note:** May not point to an existing or valid message or thread.
    pub last_message_id: Option<MessageId>,
    /// Amount of seconds a user has to wait before sending another message.
    ///
    /// Bots, as well as users with the [`BYPASS_SLOWMODE`] permission, are unaffected.
    ///
    /// [`BYPASS_SLOWMODE`]: Permissions::BYPASS_SLOWMODE
    #[doc(alias = "slowmode")]
    #[serde(default)]
    pub rate_limit_per_user: Option<NonMaxU16>,
    /// The timestamp of the time the last pinned message was pinned.
    pub last_pin_timestamp: Option<Timestamp>,
    /// Computed permissions for the invoking user in the channel, including overwrites.
    pub permissions: Option<Permissions>,
    /// Computed permissions for the bot user in the channel, including overwrites.
    pub app_permissions: Option<Permissions>,
    /// Extra information about the channel.
    #[serde(default)]
    pub flags: ChannelFlags,
}

/// Represents a partial channel from an interaction.
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object),
/// [subset specification](https://docs.discord.com/developers/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InteractionChannel {
    /// The shared fields between [`InteractionChannel`] and [`InteractionGuildThread`].
    #[serde(flatten)]
    pub base: BaseInteractionChannel,
    /// The channel Id.
    pub id: ChannelId,
    /// Sorting position of the channel. Channels with the same position are sorted by Id.
    ///
    /// The default text channel will _almost always_ have a position of `0`.
    #[serde(default)]
    pub position: u16,
    /// The topic of the channel.
    pub topic: Option<FixedString<u16>>,
    /// Whether the channel is age-restricted.
    #[serde(default)]
    pub nsfw: bool,
    /// The Id of the parent category.
    ///
    /// **Note**: This is only available for channels in a category.
    pub parent_id: Option<ChannelId>,
}

/// Represents a partial thread from an interaction.
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object),
/// [subset specification](https://docs.discord.com/developers/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InteractionGuildThread {
    /// The shared fields between [`InteractionChannel`] and [`InteractionGuildThread`].
    #[serde(flatten)]
    pub base: BaseInteractionChannel,
    /// The thread Id.
    pub id: ThreadId,
    /// The Id of the parent text channel.
    pub parent_id: ChannelId,
    /// The thread metadata.
    ///
    /// **Note**: This is only available on thread channels.
    pub thread_metadata: ThreadMetadata,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum GenericInteractionChannel {
    Channel(InteractionChannel),
    Thread(InteractionGuildThread),
}

impl GenericInteractionChannel {
    #[must_use]
    pub fn id(&self) -> GenericChannelId {
        match self {
            Self::Channel(ch) => ch.id.widen(),
            Self::Thread(th) => th.id.widen(),
        }
    }

    #[must_use]
    pub fn base(&self) -> &BaseInteractionChannel {
        match self {
            Self::Channel(ch) => &ch.base,
            Self::Thread(th) => &th.base,
        }
    }
}

impl ExtractKey<GenericChannelId> for GenericInteractionChannel {
    fn extract_key(&self) -> &GenericChannelId {
        match self {
            Self::Channel(channel) => GenericChannelId::cast_from(&channel.id.0),
            Self::Thread(thread) => GenericChannelId::cast_from(&thread.id.0),
        }
    }
}

impl<'de> serde::Deserialize<'de> for GenericInteractionChannel {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (kind, _, data) = super::extract_type(deserializer)?;

        match kind {
            10..=12 => Deserialize::deserialize(data).map(Self::Thread),
            _ => Deserialize::deserialize(data).map(Self::Channel),
        }
        .map_err(serde::de::Error::custom)
    }
}
