use super::{ChannelId, WebhookId};

/// A container for the IDs returned by following a news channel.
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#followed-channel-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FollowedChannel {
    /// The source news channel
    pub channel_id: ChannelId,
    /// The created webhook ID in the target channel
    pub webhook_id: WebhookId,
}
