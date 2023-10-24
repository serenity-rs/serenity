use futures::channel::mpsc::{TrySendError, UnboundedSender as Sender};
use tokio_tungstenite::tungstenite::Message;

use super::{ChunkGuildFilter, ShardClientMessage, ShardRunnerMessage};
#[cfg(feature = "collector")]
use crate::collector::{
    ComponentInteractionFilter,
    EventFilter,
    MessageFilter,
    ModalInteractionFilter,
    ReactionFilter,
};
use crate::gateway::{ActivityData, InterMessage};
use crate::model::prelude::*;

/// A lightweight wrapper around an mpsc sender.
///
/// This is used to cleanly communicate with a shard's respective
/// [`ShardRunner`]. This can be used for actions such as setting the activity
/// via [`Self::set_activity`] or shutting down via [`Self::shutdown_clean`].
///
/// [`ShardRunner`]: super::ShardRunner
#[derive(Clone, Debug)]
pub struct ShardMessenger {
    pub(crate) tx: Sender<InterMessage>,
}

impl ShardMessenger {
    /// Creates a new shard messenger.
    ///
    /// If you are using the [`Client`], you do not need to do this.
    ///
    /// [`Client`]: crate::Client
    #[inline]
    #[must_use]
    pub const fn new(tx: Sender<InterMessage>) -> Self {
        Self {
            tx,
        }
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large
    /// guilds (250 members+). If a guild is over 250 members, then a full
    /// member list will not be downloaded, and must instead be requested to be
    /// sent in "chunks" containing members.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each
    /// chunk only contains a partial amount of the total members.
    ///
    /// If the `cache` feature is enabled, the cache will automatically be
    /// updated with member chunks.
    ///
    /// # Examples
    ///
    /// Chunk a single guild by Id, limiting to 2000 [`Member`]s, and not
    /// specifying a query parameter:
    ///
    /// ```rust,no_run
    /// # use tokio::sync::Mutex;
    /// # use serenity::model::gateway::{GatewayIntents, ShardInfo};
    /// # use serenity::client::bridge::gateway::ChunkGuildFilter;
    /// # use serenity::gateway::Shard;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let shard_info = ShardInfo {
    /// #         id: 0,
    /// #         total: 1,
    /// #     };
    /// #     let mut shard = Shard::new(mutex.clone(), "", shard_info, GatewayIntents::all(), None).await?;
    /// #
    /// use serenity::model::id::GuildId;
    ///
    /// shard.chunk_guild(GuildId::new(81384788765712384), Some(2000), ChunkGuildFilter::None, None);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Chunk a single guild by Id, limiting to 20 members, specifying a
    /// query parameter of `"do"` and a nonce of `"request"`:
    ///
    /// ```rust,no_run
    /// # use tokio::sync::Mutex;
    /// # use serenity::model::gateway::{GatewayIntents, ShardInfo};
    /// # use serenity::client::bridge::gateway::ChunkGuildFilter;
    /// # use serenity::gateway::Shard;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let shard_info = ShardInfo {
    /// #         id: 0,
    /// #         total: 1,
    /// #     };
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", shard_info, GatewayIntents::all(), None).await?;;
    /// #
    /// use serenity::model::id::GuildId;
    ///
    /// shard.chunk_guild(
    ///     GuildId::new(81384788765712384),
    ///     Some(20),
    ///     ChunkGuildFilter::Query("do".to_owned()),
    ///     Some("request"),
    /// );
    /// #     Ok(())
    /// # }
    /// ```
    pub fn chunk_guild(
        &self,
        guild_id: GuildId,
        limit: Option<u16>,
        filter: ChunkGuildFilter,
        nonce: Option<String>,
    ) {
        drop(self.send_to_shard(ShardRunnerMessage::ChunkGuild {
            guild_id,
            limit,
            filter,
            nonce,
        }));
    }

    /// Sets the user's current activity, if any.
    ///
    /// Other presence settings are maintained.
    ///
    /// # Examples
    ///
    /// Setting the current activity to playing `"Heroes of the Storm"`:
    ///
    /// ```rust,no_run
    /// # use tokio::sync::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use serenity::model::gateway::{GatewayIntents, ShardInfo};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let shard_info = ShardInfo {
    /// #         id: 0,
    /// #         total: 1,
    /// #     };
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", shard_info, GatewayIntents::all(), None).await?;
    /// use serenity::gateway::ActivityData;
    ///
    /// shard.set_activity(Some(ActivityData::playing("Heroes of the Storm")));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn set_activity(&self, activity: Option<ActivityData>) {
        drop(self.send_to_shard(ShardRunnerMessage::SetActivity(activity)));
    }

    /// Sets the user's full presence information.
    ///
    /// Consider using the individual setters if you only need to modify one of
    /// these.
    ///
    /// # Examples
    ///
    /// Set the current user as playing `"Heroes of the Storm"` and being
    /// online:
    ///
    /// ```rust,ignore
    /// # use tokio::sync::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #
    /// #     let shard_info = ShardInfo {
    /// #         id: 0,
    /// #         total: 1,
    /// #     };
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", shard_info, None).await?;
    /// #
    /// use serenity::gateway::ActivityData;
    /// use serenity::model::user::OnlineStatus;
    ///
    /// let activity = ActivityData::playing("Heroes of the Storm");
    /// shard.set_presence(Some(activity), OnlineStatus::Online);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn set_presence(&self, activity: Option<ActivityData>, mut status: OnlineStatus) {
        if status == OnlineStatus::Offline {
            status = OnlineStatus::Invisible;
        }

        drop(self.send_to_shard(ShardRunnerMessage::SetPresence(activity, status)));
    }

    /// Sets the user's current online status.
    ///
    /// Note that [`Offline`] is not a valid online status, so it is
    /// automatically converted to [`Invisible`].
    ///
    /// Other presence settings are maintained.
    ///
    /// # Examples
    ///
    /// Setting the current online status for the shard to [`DoNotDisturb`].
    ///
    /// ```rust,no_run
    /// # use tokio::sync::Mutex;
    /// # use serenity::gateway::Shard;
    /// # use serenity::model::gateway::{GatewayIntents, ShardInfo};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mutex = Arc::new(Mutex::new("".to_string()));
    /// #     let shard_info = ShardInfo {
    /// #         id: 0,
    /// #         total: 1,
    /// #     };
    /// #
    /// #     let mut shard = Shard::new(mutex.clone(), "", shard_info, GatewayIntents::all(), None).await?;
    /// #
    /// use serenity::model::user::OnlineStatus;
    ///
    /// shard.set_status(OnlineStatus::DoNotDisturb);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`DoNotDisturb`]: OnlineStatus::DoNotDisturb
    /// [`Invisible`]: OnlineStatus::Invisible
    /// [`Offline`]: OnlineStatus::Offline
    pub fn set_status(&self, mut online_status: OnlineStatus) {
        if online_status == OnlineStatus::Offline {
            online_status = OnlineStatus::Invisible;
        }

        drop(self.send_to_shard(ShardRunnerMessage::SetStatus(online_status)));
    }

    /// Shuts down the websocket by attempting to cleanly close the
    /// connection.
    pub fn shutdown_clean(&self) {
        drop(self.send_to_shard(ShardRunnerMessage::Close(1000, None)));
    }

    /// Sends a raw message over the WebSocket.
    ///
    /// The given message is not mutated in any way, and is sent as-is.
    ///
    /// You should only use this if you know what you're doing. If you're
    /// wanting to, for example, send a presence update, prefer the usage of
    /// the [`Self::set_presence`] method.
    pub fn websocket_message(&self, message: Message) {
        drop(self.send_to_shard(ShardRunnerMessage::Message(message)));
    }

    /// Sends a message to the shard.
    ///
    /// # Errors
    ///
    /// Returns a [`TrySendError`] if the shard's receiver was closed.
    #[inline]
    pub fn send_to_shard(&self, msg: ShardRunnerMessage) -> Result<(), TrySendError<InterMessage>> {
        self.tx.unbounded_send(InterMessage::Client(ShardClientMessage::Runner(Box::new(msg))))
    }

    /// Sets a new filter for an event collector.
    #[inline]
    #[cfg(feature = "collector")]
    pub fn set_event_filter(&self, collector: EventFilter) {
        drop(self.send_to_shard(ShardRunnerMessage::SetEventFilter(collector)));
    }

    /// Sets a new filter for a message collector.
    #[inline]
    #[cfg(feature = "collector")]
    pub fn set_message_filter(&self, collector: MessageFilter) {
        drop(self.send_to_shard(ShardRunnerMessage::SetMessageFilter(collector)));
    }

    /// Sets a new filter for a reaction collector.
    #[cfg(feature = "collector")]
    pub fn set_reaction_filter(&self, collector: ReactionFilter) {
        drop(self.send_to_shard(ShardRunnerMessage::SetReactionFilter(collector)));
    }

    /// Sets a new filter for a component interaction collector.
    #[cfg(feature = "collector")]
    pub fn set_component_interaction_filter(&self, collector: ComponentInteractionFilter) {
        drop(self.send_to_shard(ShardRunnerMessage::SetComponentInteractionFilter(collector)));
    }

    /// Sets a new filter for a modal interaction collector.
    #[cfg(feature = "collector")]
    pub fn set_modal_interaction_filter(&self, collector: ModalInteractionFilter) {
        drop(self.send_to_shard(ShardRunnerMessage::SetModalInteractionFilter(collector)));
    }
}

impl AsRef<ShardMessenger> for ShardMessenger {
    fn as_ref(&self) -> &ShardMessenger {
        self
    }
}
