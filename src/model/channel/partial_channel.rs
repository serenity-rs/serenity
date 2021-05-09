use crate::model::channel::ChannelType;
use crate::model::id::ChannelId;
use crate::model::Permissions;

/// A container for any partial channel.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PartialChannel {
    /// The channel Id.
    pub id: ChannelId,
    /// The channel name.
    pub name: String,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The channel permissions.
    pub permissions: Option<Permissions>,
}
