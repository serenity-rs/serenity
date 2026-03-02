use std::sync::Arc;

use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender as Sender;

use crate::builder::DataUri;
#[cfg(feature = "cache")]
pub use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::gateway::CollectorCallback;
use crate::gateway::{
    ActivityData,
    ChunkGuildFilter,
    ShardManagerMessage,
    ShardRunnerInfo,
    ShardRunnerMessage,
};
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;

/// A general utility struct provided on event dispatches.
///
/// The [`Context`] helps with dealing with the current "context" of the event dispatch. It also
/// acts as a general high-level interface over the low-level [`http`] module, plus the associated
/// [`Shard`] which received the event.
///
/// The context contains "shortcuts", like for interacting with the shard. Methods like
/// [`Self::set_activity`] will unlock the shard and perform an update for you to save a bit of
/// work.
///
/// A context will only live for the event it was dispatched for. After the event handler finished,
/// it is destroyed and will not be re-used.
///
/// [`Shard`]: crate::gateway::Shard
/// [`http`]: crate::http
#[derive(Clone)]
pub struct Context {
    /// A clone of [`Client::data`]. Refer to its documentation for more information.
    ///
    /// [`Client::data`]: super::Client::data
    pub(crate) data: Arc<dyn std::any::Any + Send + Sync>,
    /// The channel to communicate with the shard runner.
    pub(crate) shard: Sender<ShardRunnerMessage>,
    /// The channel to communicate with the shard manager.
    pub(crate) manager: Sender<ShardManagerMessage>,
    /// The ID of the shard this context is related to.
    pub shard_id: ShardId,
    pub http: Arc<Http>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
    /// Metadata about the initialised shards, and their control channels.
    pub runners: Arc<DashMap<ShardId, (ShardRunnerInfo, Sender<ShardRunnerMessage>)>>,
    #[cfg(feature = "collector")]
    pub(crate) collectors: Arc<parking_lot::RwLock<Vec<CollectorCallback>>>,
}

impl CacheHttp for Context {
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}

impl Context {
    /// A container for a data type that can be used across contexts.
    ///
    /// The purpose of the data field is to be accessible and persistent across contexts; that is,
    /// data can be modified by one context, and will persist through the future and be accessible
    /// through other contexts. This is useful for anything that should "live" through the program:
    /// counters, database connections, custom user caches, etc.
    ///
    /// # Panics
    /// Panics if the generic provided is not equal to the type provided in [`ClientBuilder::data`].
    ///
    /// [`ClientBuilder::data`]: super::ClientBuilder::data
    #[must_use]
    pub fn data<Data: Send + Sync + 'static>(&self) -> Arc<Data> {
        Arc::clone(&self.data)
            .downcast()
            .expect("Type provided to Context should be the same as ClientBuilder::data.")
    }

    /// A version of [`Self::data`] which returns a reference to the Data.
    ///
    /// This is useful if you need to borrow `Data` with the lifetime of `Context`, but otherwise
    /// [`Self::data`] should be used.
    #[must_use]
    #[expect(clippy::needless_lifetimes, reason = "Easier to understand when explicitly written")]
    pub fn data_ref<'a, Data: Send + Sync + 'static>(&'a self) -> &'a Data {
        self.data
            .downcast_ref()
            .expect("Type provided to Context should be the same as ClientBuilder::data.")
    }

    /// Sets the current user as being [`Online`]. This maintains the current activity.
    ///
    /// [`Online`]: OnlineStatus::Online
    pub fn online(&self) {
        self.set_status(OnlineStatus::Online);
    }

    /// Sets the current user as being [`Idle`]. This maintains the current activity.
    ///
    /// [`Idle`]: OnlineStatus::Idle
    pub fn idle(&self) {
        self.set_status(OnlineStatus::Idle);
    }

    /// Sets the current user as being [`DoNotDisturb`]. This maintains the current activity.
    ///
    ///
    /// [`DoNotDisturb`]: OnlineStatus::DoNotDisturb
    pub fn dnd(&self) {
        self.set_status(OnlineStatus::DoNotDisturb);
    }

    /// Sets the current user as being [`Invisible`]. This maintains the current activity.
    ///
    /// [`Invisible`]: OnlineStatus::Invisible
    pub fn invisible(&self) {
        self.set_status(OnlineStatus::Invisible);
    }

    /// Sets the user's current online status.
    ///
    /// Note that [`Offline`] is not a valid online status, so it is automatically converted to
    /// [`Invisible`].
    ///
    /// [`Invisible`]: OnlineStatus::Invisible
    /// [`Offline`]: OnlineStatus::Offline
    pub fn set_status(&self, mut online_status: OnlineStatus) {
        if online_status == OnlineStatus::Offline {
            online_status = OnlineStatus::Invisible;
        }

        self.send_to_shard(ShardRunnerMessage::SetPresence {
            activity: None,
            status: Some(online_status),
        });
    }

    /// "Resets" the current user's presence, by setting the activity to [`None`] and the online
    /// status to [`Online`].
    ///
    /// Use [`Self::set_presence`] for fine-grained control over individual details.
    ///
    /// [`Online`]: OnlineStatus::Online
    pub fn reset_presence(&self) {
        self.set_presence(None, OnlineStatus::Online);
    }

    /// Sets the current activity.
    pub fn set_activity(&self, activity: Option<ActivityData>) {
        self.send_to_shard(ShardRunnerMessage::SetPresence {
            activity: Some(activity),
            status: None,
        });
    }

    /// Sets the current user's presence, providing all fields to be passed.
    pub fn set_presence(&self, activity: Option<ActivityData>, mut status: OnlineStatus) {
        if status == OnlineStatus::Offline {
            status = OnlineStatus::Invisible;
        }

        self.send_to_shard(ShardRunnerMessage::SetPresence {
            activity: Some(activity),
            status: Some(status),
        });
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large guilds. If a guild is
    /// large enough, then a full member list will not be provided upon connection, and must
    /// instead be requested directly. The full list will be sent in "chunks" until all members
    /// matching the request have been sent.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each chunk only contains
    /// a partial amount of the total members.
    ///
    /// # Examples
    ///
    /// Chunk a single guild, limiting to 2000 [`Member`]s, and not specifying a query
    /// parameter:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// # use serenity::gateway::client::FullEvent;
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # use serenity::all::GuildId;
    ///
    /// # struct Handler;
    /// #
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
    ///         match event {
    ///             FullEvent::Ready {
    ///                 ..
    ///             } => {
    ///                 ctx.chunk_guild(
    ///                     GuildId::new(81384788765712384),
    ///                     Some(2000),
    ///                     false,
    ///                     ChunkGuildFilter::None,
    ///                     None,
    ///                 );
    ///             },
    ///             _ => {},
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Chunk a single guild by Id, limiting to 20 members, specifying a query parameter of `"do"`
    /// and a nonce of `"request"`:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// # use serenity::gateway::client::FullEvent;
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # use serenity::all::GuildId;
    ///
    /// # struct Handler;
    /// #
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
    ///         match event {
    ///             FullEvent::Ready {
    ///                 ..
    ///             } => {
    ///                 ctx.chunk_guild(
    ///                     GuildId::new(81384788765712384),
    ///                     Some(20),
    ///                     false,
    ///                     ChunkGuildFilter::Query("do".to_owned()),
    ///                     Some("request".to_string()),
    ///                 );
    ///             },
    ///             _ => {},
    ///         }
    ///     }
    /// }
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

    /// Indicates to the gateway that the client wants to join, move, or disconnect from a voice
    /// channel.
    #[cfg(feature = "voice")]
    pub fn update_voice_state(
        &self,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
    ) {
        self.send_to_shard(ShardRunnerMessage::UpdateVoiceState {
            guild_id,
            channel_id,
            self_mute,
            self_deaf,
        });
    }

    /// Sends a message to the shard.
    fn send_to_shard(&self, msg: ShardRunnerMessage) {
        if let Err(e) = self.shard.unbounded_send(msg) {
            tracing::warn!("failed to send ShardRunnerMessage to shard: {}", e);
        }
    }

    /// Sends a message back to the shard manager to shutdown all currently running shards,
    /// including this one.
    pub fn shutdown_all(&self) {
        if let Err(e) = self.manager.unbounded_send(ShardManagerMessage::Quit(Ok(()))) {
            tracing::warn!("failed to send shutdown request to shard manager: {}", e);
        }
    }

    /// Gets all emojis for the current application.
    ///
    /// # Errors
    ///
    /// Returns an error if the Application ID is not known.
    pub async fn get_application_emojis(&self) -> Result<Vec<Emoji>> {
        self.http.get_application_emojis().await
    }

    /// Gets information about an application emoji.
    ///
    /// # Errors
    ///
    /// Returns an error if the emoji does not exist.
    pub async fn get_application_emoji(&self, emoji_id: EmojiId) -> Result<Emoji> {
        self.http.get_application_emoji(emoji_id).await
    }

    /// Creates an application emoji with a name and base64-encoded image.
    ///
    /// # Errors
    ///
    /// See [`GuildId::create_emoji`] for information about name and filesize requirements. This
    /// method will error if said requirements are not met.
    pub async fn create_application_emoji(&self, name: &str, image: DataUri<'_>) -> Result<Emoji> {
        #[derive(serde::Serialize)]
        struct CreateEmoji<'a> {
            name: &'a str,
            image: DataUri<'a>,
        }

        let body = CreateEmoji {
            name,
            image,
        };

        self.http.create_application_emoji(&body).await
    }

    /// Changes the name of an application emoji.
    ///
    /// # Errors
    ///
    /// Returns an error if the emoji does not exist.
    pub async fn edit_application_emoji(&self, emoji_id: EmojiId, name: &str) -> Result<Emoji> {
        #[derive(serde::Serialize)]
        struct EditEmoji<'a> {
            name: &'a str,
        }

        let body = EditEmoji {
            name,
        };

        self.http.edit_application_emoji(emoji_id, &body).await
    }

    /// Deletes an application emoji.
    ///
    /// # Errors
    ///
    /// Returns an error if the emoji does not exist.
    pub async fn delete_application_emoji(&self, emoji_id: EmojiId) -> Result<()> {
        self.http.delete_application_emoji(emoji_id).await
    }
}
