use crate::internal::prelude::*;
use crate::model::channel::{ChannelType, ThreadMetadata};
use crate::model::id::{ChannelId, WebhookId};
use crate::model::Permissions;

/// A container for any partial channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object),
/// [subset specification](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialChannel {
    /// The channel Id.
    pub id: ChannelId,
    /// The channel name.
    pub name: Option<FixedString>,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The channel permissions.
    pub permissions: Option<Permissions>,
    /// The thread metadata.
    ///
    /// **Note**: This is only available on thread channels.
    pub thread_metadata: Option<ThreadMetadata>,
    /// The Id of the parent category for a channel, or of the parent text channel for a thread.
    ///
    /// **Note**: This is only available on thread channels.
    pub parent_id: Option<ChannelId>,
}

impl ExtractKey<ChannelId> for PartialChannel {
    fn extract_key(&self) -> &ChannelId {
        &self.id
    }
}

/// A container for the IDs returned by following a news channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#followed-channel-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FollowedChannel {
    /// The source news channel
    pub channel_id: ChannelId,
    /// The created webhook ID in the target channel
    pub webhook_id: WebhookId,
}
