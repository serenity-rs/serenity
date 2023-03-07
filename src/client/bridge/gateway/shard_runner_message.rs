use async_tungstenite::tungstenite::Message;

#[cfg(feature = "collector")]
use crate::collector::{
    ComponentInteractionFilter,
    EventFilter,
    MessageFilter,
    ModalInteractionFilter,
    ReactionFilter,
};
use crate::model::gateway::Activity;
use crate::model::id::{GuildId, UserId};
use crate::model::user::OnlineStatus;

#[derive(Clone, Debug)]
pub enum ChunkGuildFilter {
    /// Returns all members of the guilds specified. Requires GUILD_MEMBERS intent.
    None,
    /// A common username prefix filter for the members returned.
    Query(String),
    /// A set of exact user IDs to query for.
    UserIds(Vec<UserId>),
}

/// A message to send from a shard over a WebSocket.
// Once we can use `Box` as part of a pattern, we will reconsider boxing.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ShardRunnerMessage {
    /// Indicates that the client is to send a member chunk message.
    ChunkGuild {
        /// The IDs of the [`Guild`] to chunk.
        ///
        /// [`Guild`]: crate::model::guild::Guild
        guild_id: GuildId,
        /// The maximum number of members to receive [`GuildMembersChunkEvent`]s
        /// for.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        limit: Option<u16>,
        /// A filter to apply to the returned members.
        filter: ChunkGuildFilter,
        /// Optional nonce to identify [`GuildMembersChunkEvent`] responses.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        nonce: Option<String>,
    },
    /// Indicates that the client is to close with the given status code and
    /// reason.
    ///
    /// You should rarely - if _ever_ - need this, but the option is available.
    /// Prefer to use the [`ShardManager`] to shutdown WebSocket clients if you
    /// are intending to send a 1000 close code.
    ///
    /// [`ShardManager`]: super::ShardManager
    Close(u16, Option<String>),
    /// Indicates that the client is to send a custom WebSocket message.
    Message(Message),
    /// Indicates that the client is to update the shard's presence's activity.
    SetActivity(Option<Activity>),
    /// Indicates that the client is to update the shard's presence in its
    /// entirety.
    SetPresence(OnlineStatus, Option<Activity>),
    /// Indicates that the client is to update the shard's presence's status.
    SetStatus(OnlineStatus),
    /// Sends a new filter for events to the shard.
    #[cfg(feature = "collector")]
    SetEventFilter(EventFilter),
    /// Sends a new filter for messages to the shard.
    #[cfg(feature = "collector")]
    SetMessageFilter(MessageFilter),
    /// Sends a new filter for reactions to the shard.
    #[cfg(feature = "collector")]
    SetReactionFilter(ReactionFilter),
    /// Sends a new filter for component interactions to the shard.
    #[cfg(feature = "collector")]
    SetComponentInteractionFilter(ComponentInteractionFilter),
    /// Sends a new filter for modal interactions to the shard.
    #[cfg(feature = "collector")]
    SetModalInteractionFilter(ModalInteractionFilter),
}
