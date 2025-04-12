use crate::model::prelude::*;

/// Represents the shared fields between a [`InteractionChannel`] and a [`InteractionGuildThread`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseInteractionChannel {
    /// The channel name.
    pub name: Option<FixedString>,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The channel permissions.
    pub permissions: Option<Permissions>,
}

/// Represents a partial channel from an interaction.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object),
/// [subset specification](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InteractionChannel {
    /// The shared fields between [`InteractionChannel`] and [`InteractionGuildThread`].
    #[serde(flatten)]
    pub base: BaseInteractionChannel,
    /// The channel Id.
    pub id: ChannelId,
}

/// Represents a partial thread from an interaction.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object),
/// [subset specification](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InteractionGuildThread {
    /// The shared fields between [`InteractionChannel`] and [`InteractionGuildThread`].
    #[serde(flatten)]
    pub base: BaseInteractionChannel,
    /// The thread Id.
    pub id: ThreadId,
    /// The thread metadata.
    ///
    /// **Note**: This is only available on thread channels.
    pub thread_metadata: ThreadMetadata,
    /// The parent text channel.
    pub parent_id: ChannelId,
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
        let (kind, data) = super::extract_type(deserializer)?;

        match kind {
            10..=12 => Deserialize::deserialize(data).map(Self::Thread),
            _ => Deserialize::deserialize(data).map(Self::Channel),
        }
        .map_err(serde::de::Error::custom)
    }
}
