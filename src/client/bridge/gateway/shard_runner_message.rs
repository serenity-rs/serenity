use crate::model::{
    gateway::Activity,
    id::GuildId,
    user::OnlineStatus,
};
use tungstenite::Message;

/// A message to send from a shard over a WebSocket.
// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ShardRunnerMessage {
    /// Indicates that the client is to send a member chunk message.
    ChunkGuilds {
        /// The IDs of the [`Guild`]s to chunk.
        ///
        /// [`Guild`]: ../../../model/guild/struct.Guild.html
        guild_ids: Vec<GuildId>,
        /// The maximum number of members to receive [`GuildMembersChunkEvent`]s
        /// for.
        ///
        /// [`GuildMembersChunkEvent`]: ../../../model/event/struct.GuildMembersChunkEvent.html
        limit: Option<u16>,
        /// Text to filter members by.
        ///
        /// For example, a query of `"s"` will cause only [`Member`]s whose
        /// usernames start with `"s"` to be chunked.
        ///
        /// [`Member`]: ../../../model/guild/struct.Member.html
        query: Option<String>,
    },
    /// Indicates that the client is to close with the given status code and
    /// reason.
    ///
    /// You should rarely - if _ever_ - need this, but the option is available.
    /// Prefer to use the [`ShardManager`] to shutdown WebSocket clients if you
    /// are intending to send a 1000 close code.
    ///
    /// [`ShardManager`]: struct.ShardManager.html
    Close(u16, Option<String>),
    /// Indicates that the client is to send a custom WebSocket message.
    Message(Message),
    /// Indicates that the client is to update the shard's presence's activity.
    SetActivity(Option<Activity>),
    /// Indicates that the client is to update the shard's presence in its
    /// entirity.
    SetPresence(OnlineStatus, Option<Activity>),
    /// Indicates that the client is to update the shard's presence's status.
    SetStatus(OnlineStatus),
}
