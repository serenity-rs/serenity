#[cfg(feature = "collector")]
use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender as Sender;
use tokio_tungstenite::tungstenite::Message;

#[cfg(feature = "collector")]
use super::CollectorCallback;
use super::{ChunkGuildFilter, ShardRunner, ShardRunnerMessage};
use crate::gateway::ActivityData;
use crate::model::prelude::*;

/// A handle to a [`ShardRunner`].
///
/// This is used to cleanly communicate with a shard's respective [`ShardRunner`]. This can be used
/// for actions such as setting the activity via [`Self::set_activity`] or shutting down via
/// [`Self::shutdown_clean`].
///
/// [`ShardRunner`]: super::ShardRunner
#[derive(Clone, Debug)]
pub struct ShardMessenger {
    pub(crate) tx: Sender<ShardRunnerMessage>,
    #[cfg(feature = "collector")]
    pub(crate) collectors: Arc<parking_lot::RwLock<Vec<CollectorCallback>>>,
}

impl ShardMessenger {
    /// Creates a new shard messenger.
    ///
    /// If you are using the [`Client`], you do not need to do this.
    ///
    /// [`Client`]: crate::Client
    #[must_use]
    pub fn new(shard: &ShardRunner) -> Self {
        Self {
            tx: shard.runner_tx(),
            #[cfg(feature = "collector")]
            collectors: Arc::clone(&shard.collectors),
        }
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large guilds (250 members+).
    /// If a guild is over 250 members, then a full member list will not be downloaded, and must
    /// instead be requested to be sent in "chunks" containing members.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each chunk only contains
    /// a partial amount of the total members.
    ///
    /// If the `cache` feature is enabled, the cache will automatically be updated with member
    /// chunks.
    ///
    /// # Examples
    ///
    /// Chunk a single guild by Id, limiting to 2000 [`Member`]s, and not specifying a query
    /// parameter:
    ///
    /// ```rust,no_run
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # async fn run(mut shard: Shard) -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::id::GuildId;
    ///
    /// shard.chunk_guild(
    ///     GuildId::new(81384788765712384),
    ///     Some(2000),
    ///     false,
    ///     ChunkGuildFilter::None,
    ///     None,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Chunk a single guild by Id, limiting to 20 members, specifying a query parameter of `"do"`
    /// and a nonce of `"request"`:
    ///
    /// ```rust,no_run
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # async fn run(mut shard: Shard) -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::id::GuildId;
    ///
    /// shard.chunk_guild(
    ///     GuildId::new(81384788765712384),
    ///     Some(20),
    ///     false,
    ///     ChunkGuildFilter::Query("do".to_owned()),
    ///     Some("request"),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn chunk_guild(
        &self,
        guild_id: GuildId,
        limit: Option<u16>,
        presences: bool,
        filter: ChunkGuildFilter,
        nonce: Option<String>,
    ) {
        self.send_to_shard(ShardRunnerMessage::ChunkGuild {
            guild_id,
            limit,
            presences,
            filter,
            nonce,
        });
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
    /// # use serenity::gateway::Shard;
    /// # async fn run(mut shard: Shard) -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::gateway::ActivityData;
    ///
    /// shard.set_activity(Some(ActivityData::playing("Heroes of the Storm")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_activity(&self, activity: Option<ActivityData>) {
        self.send_to_shard(ShardRunnerMessage::SetActivity(activity));
    }

    /// Sets the user's full presence information.
    ///
    /// Consider using the individual setters if you only need to modify one of these.
    ///
    /// # Examples
    ///
    /// Set the current user as playing `"Heroes of the Storm"` and being online:
    ///
    /// ```rust,no_run
    /// # use serenity::gateway::Shard;
    /// # async fn run(mut shard: Shard) -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::gateway::ActivityData;
    /// use serenity::model::user::OnlineStatus;
    ///
    /// let activity = ActivityData::playing("Heroes of the Storm");
    /// shard.set_presence(Some(activity), OnlineStatus::Online);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_presence(&self, activity: Option<ActivityData>, mut status: OnlineStatus) {
        if status == OnlineStatus::Offline {
            status = OnlineStatus::Invisible;
        }

        self.send_to_shard(ShardRunnerMessage::SetPresence(activity, status));
    }

    /// Sets the user's current online status.
    ///
    /// Note that [`Offline`] is not a valid online status, so it is automatically converted to
    /// [`Invisible`].
    ///
    /// Other presence settings are maintained.
    ///
    /// # Examples
    ///
    /// Setting the current online status for the shard to [`DoNotDisturb`].
    ///
    /// ```rust,no_run
    /// # use serenity::gateway::Shard;
    /// # async fn run(mut shard: Shard) -> Result<(), Box<dyn std::error::Error>> {
    /// use serenity::model::user::OnlineStatus;
    ///
    /// shard.set_status(OnlineStatus::DoNotDisturb);
    /// # Ok(())
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

        self.send_to_shard(ShardRunnerMessage::SetStatus(online_status));
    }

    /// Shuts down the websocket by attempting to cleanly close the connection.
    pub fn shutdown_clean(&self) {
        self.send_to_shard(ShardRunnerMessage::Close(1000, None));
    }

    /// Sends a raw message over the WebSocket.
    ///
    /// The given message is not mutated in any way, and is sent as-is.
    ///
    /// You should only use this if you know what you're doing. If you're wanting to, for example,
    /// send a presence update, prefer the usage of the [`Self::set_presence`] method.
    pub fn websocket_message(&self, message: Message) {
        self.send_to_shard(ShardRunnerMessage::Message(message));
    }

    /// Sends a message to the shard.
    pub fn send_to_shard(&self, msg: ShardRunnerMessage) {
        if let Err(e) = self.tx.unbounded_send(msg) {
            tracing::warn!("failed to send ShardRunnerMessage to shard: {}", e);
        }
    }

    #[cfg(feature = "collector")]
    pub fn add_collector(&self, collector: CollectorCallback) {
        self.collectors.write().push(collector);
    }
}
