use crate::model::channel::ChannelType;
use crate::model::id::{ChannelId, WebhookId};
use crate::model::Permissions;

/// A container for any partial channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object), [subset specification](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PartialChannel {
    /// The channel Id.
    pub id: ChannelId,
    /// The channel name.
    pub name: Option<String>,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The channel permissions.
    pub permissions: Option<Permissions>,
}

/// A container for the IDs returned by following a news channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#followed-channel-object).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FollowedChannel {
    /// The source news channel
    channel_id: ChannelId,
    /// The created webhook ID in the target channel
    webhook_id: WebhookId,
}
