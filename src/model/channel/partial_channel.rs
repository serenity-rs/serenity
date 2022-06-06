use crate::model::channel::ChannelType;
use crate::model::id::{ChannelId, WebhookId};
use crate::model::Permissions;

/// A container for any partial channel.
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FollowedChannel {
    /// The source news channel
    channel_id: ChannelId,
    /// The created webhook ID in the target channel
    webhook_id: WebhookId,
}
