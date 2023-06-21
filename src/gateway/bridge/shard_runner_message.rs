use tokio_tungstenite::tungstenite::Message;

use super::ShardId;
use crate::gateway::{ActivityData, ChunkGuildFilter};
use crate::model::id::GuildId;
use crate::model::user::OnlineStatus;

/// A message to send from a shard over a WebSocket.
#[derive(Debug)]
pub enum ShardRunnerMessage {
    /// Indicator that a shard should be restarted.
    Restart(ShardId),
    /// Indicator that a shard should be fully shutdown without bringing it
    /// back up.
    Shutdown(ShardId, u16),
    /// Indicates that the client is to send a member chunk message.
    ChunkGuild {
        /// The IDs of the [`Guild`] to chunk.
        ///
        /// [`Guild`]: crate::model::guild::Guild
        guild_id: GuildId,
        /// The maximum number of members to receive [`GuildMembersChunkEvent`]s for.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        limit: Option<u16>,
        /// Used to specify if we want the presences of the matched members.
        ///
        /// Requires [`crate::model::gateway::GatewayIntents::GUILD_PRESENCES`].
        presences: bool,
        /// A filter to apply to the returned members.
        filter: ChunkGuildFilter,
        /// Optional nonce to identify [`GuildMembersChunkEvent`] responses.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        nonce: Option<String>,
    },
    /// Indicates that the client is to close with the given status code and reason.
    ///
    /// You should rarely - if _ever_ - need this, but the option is available. Prefer to use the
    /// [`ShardManager`] to shutdown WebSocket clients if you are intending to send a 1000 close
    /// code.
    ///
    /// [`ShardManager`]: super::ShardManager
    Close(u16, Option<String>),
    /// Indicates that the client is to send a custom WebSocket message.
    Message(Message),
    /// Indicates that the client is to update the shard's presence's activity.
    SetActivity(Option<ActivityData>),
    /// Indicates that the client is to update the shard's presence in its entirety.
    SetPresence(Option<ActivityData>, OnlineStatus),
    /// Indicates that the client is to update the shard's presence's status.
    SetStatus(OnlineStatus),
}
