//! Representations of voice information.

use super::id::{ChannelId, UserId};

/// Information about an available voice region.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceRegion {
    /// Whether it is a custom voice region, which is used for events.
    pub custom: bool,
    /// Whether it is a deprecated voice region, which you should avoid using.
    pub deprecated: bool,
    /// The internal Id of the voice region.
    pub id: String,
    /// A recognizable name of the location of the voice region.
    pub name: String,
    /// Whether the voice region is optimal for use by the current user.
    pub optional: bool,
    /// an example hostname.
    pub sample_hostname: String,
    /// An example port.
    pub sample_port: u64,
    /// Indicator of whether the voice region is only for VIP guilds.
    pub vip: bool,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

/// A user's state within a voice channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceState {
    pub channel_id: Option<ChannelId>,
    pub deaf: bool,
    pub mute: bool,
    pub self_deaf: bool,
    pub self_mute: bool,
    pub self_stream: Option<bool>,
    pub session_id: String,
    pub suppress: bool,
    pub token: Option<String>,
    pub user_id: UserId,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}
