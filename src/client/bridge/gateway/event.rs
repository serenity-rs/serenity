//! A collection of events created by the client, not a part of the Discord API
//! itself.

use super::ShardId;
use crate::gateway::ConnectionStage;

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub(crate) enum ClientEvent {
    ShardStageUpdate(ShardStageUpdateEvent),
}

/// An event denoting that a shard's connection stage was changed.
///
/// # Examples
///
/// This might happen when a shard changes from [`ConnectionStage::Identifying`]
/// to [`ConnectionStage::Connected`].
///
/// [`ConnectionStage::Connected`]: ../../../../gateway/enum.ConnectionStage.html#variant.Connected
/// [`ConnectionStage::Identifying`]: ../../../../gateway/enum.ConnectionStage.html#variant.Identifying
#[derive(Clone, Debug)]
pub struct ShardStageUpdateEvent {
    /// The new connection stage.
    pub new: ConnectionStage,
    /// The old connection stage.
    pub old: ConnectionStage,
    /// The ID of the shard that had its connection stage change.
    pub shard_id: ShardId,
}
