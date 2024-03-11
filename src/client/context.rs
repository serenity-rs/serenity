use std::fmt;
use std::sync::Arc;

use tokio::sync::RwLock;
use typemap_rev::TypeMap;

#[cfg(feature = "cache")]
pub use crate::cache::Cache;
use crate::gateway::ActivityData;
#[cfg(feature = "gateway")]
use crate::gateway::{ShardMessenger, ShardRunner};
use crate::http::Http;
use crate::model::prelude::*;

/// The context is a general utility struct provided on event dispatches, which helps with dealing
/// with the current "context" of the event dispatch. The context also acts as a general high-level
/// interface over the associated [`Shard`] which received the event, or the low-level [`http`]
/// module.
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
    pub data: Arc<RwLock<TypeMap>>,
    /// The messenger to communicate with the shard runner.
    pub shard: ShardMessenger,
    /// The ID of the shard this context is related to.
    pub shard_id: ShardId,
    pub http: Arc<Http>,
    #[cfg(feature = "cache")]
    pub cache: Arc<Cache>,
}

// Used by the #[cfg_attr(feature = "tracing_instrument", instrument)] macro on
// client::dispatch::handle_event
impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("shard", &self.shard)
            .field("shard_id", &self.shard_id)
            .finish_non_exhaustive()
    }
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    #[cfg(feature = "gateway")]
    pub(crate) fn new(
        data: Arc<RwLock<TypeMap>>,
        runner: &ShardRunner,
        shard_id: ShardId,
        http: Arc<Http>,
        #[cfg(feature = "cache")] cache: Arc<Cache>,
    ) -> Context {
        Context {
            shard: ShardMessenger::new(runner),
            shard_id,
            data,
            http,
            #[cfg(feature = "cache")]
            cache,
        }
    }

    #[cfg(all(not(feature = "cache"), not(feature = "gateway")))]
    pub fn easy(data: Arc<RwLock<TypeMap>>, shard_id: u32, http: Arc<Http>) -> Context {
        Context {
            shard_id,
            data,
            http,
        }
    }

    /// Sets the current user as being [`Online`]. This maintains the current activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being online on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!online" {
    ///             ctx.online();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`Online`]: OnlineStatus::Online
    #[cfg(feature = "gateway")]
    #[inline]
    pub fn online(&self) {
        self.shard.set_status(OnlineStatus::Online);
    }

    /// Sets the current user as being [`Idle`]. This maintains the current activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being idle on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!idle" {
    ///             ctx.idle();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`Idle`]: OnlineStatus::Idle
    #[cfg(feature = "gateway")]
    #[inline]
    pub fn idle(&self) {
        self.shard.set_status(OnlineStatus::Idle);
    }

    /// Sets the current user as being [`DoNotDisturb`]. This maintains the current activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being Do Not Disturb on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!dnd" {
    ///             ctx.dnd();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`DoNotDisturb`]: OnlineStatus::DoNotDisturb
    #[cfg(feature = "gateway")]
    #[inline]
    pub fn dnd(&self) {
        self.shard.set_status(OnlineStatus::DoNotDisturb);
    }

    /// Sets the current user as being [`Invisible`]. This maintains the current activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being invisible on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!invisible" {
    ///             ctx.invisible();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`Invisible`]: OnlineStatus::Invisible
    #[cfg(feature = "gateway")]
    #[inline]
    pub fn invisible(&self) {
        self.shard.set_status(OnlineStatus::Invisible);
    }

    /// "Resets" the current user's presence, by setting the activity to [`None`] and the online
    /// status to [`Online`].
    ///
    /// Use [`Self::set_presence`] for fine-grained control over individual details.
    ///
    /// # Examples
    ///
    /// Reset the current user's presence on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!reset_presence" {
    ///             ctx.reset_presence();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`Event::Resumed`]: crate::model::event::Event::Resumed
    /// [`Online`]: OnlineStatus::Online
    #[cfg(feature = "gateway")]
    #[inline]
    pub fn reset_presence(&self) {
        self.shard.set_presence(None, OnlineStatus::Online);
    }

    /// Sets the current activity.
    ///
    /// # Examples
    ///
    /// Create a command named `~setgame` that accepts a name of a game to be playing:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// # struct Handler;
    ///
    /// use serenity::gateway::ActivityData;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         let mut args = msg.content.splitn(2, ' ');
    ///
    ///         if let (Some("~setgame"), Some(game)) = (args.next(), args.next()) {
    ///             ctx.set_activity(Some(ActivityData::playing(game)));
    ///         }
    ///     }
    /// }
    /// ```
    #[cfg(feature = "gateway")]
    #[inline]
    pub fn set_activity(&self, activity: Option<ActivityData>) {
        self.shard.set_activity(activity);
    }

    /// Sets the current user's presence, providing all fields to be passed.
    ///
    /// # Examples
    ///
    /// Setting the current user as having no activity and being [`Idle`]:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn ready(&self, ctx: Context, _: Ready) {
    ///         use serenity::model::user::OnlineStatus;
    ///
    ///         ctx.set_presence(None, OnlineStatus::Idle);
    ///     }
    /// }
    /// ```
    ///
    /// Setting the current user as playing `"Heroes of the Storm"`, while being [`DoNotDisturb`]:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// # struct Handler;
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn ready(&self, context: Context, _: Ready) {
    ///         use serenity::gateway::ActivityData;
    ///         use serenity::model::user::OnlineStatus;
    ///
    ///         let activity = ActivityData::playing("Heroes of the Storm");
    ///         let status = OnlineStatus::DoNotDisturb;
    ///
    ///         context.set_presence(Some(activity), status);
    ///     }
    /// }
    /// ```
    ///
    /// [`DoNotDisturb`]: OnlineStatus::DoNotDisturb
    /// [`Idle`]: OnlineStatus::Idle
    #[cfg(feature = "gateway")]
    #[inline]
    pub fn set_presence(&self, activity: Option<ActivityData>, status: OnlineStatus) {
        self.shard.set_presence(activity, status);
    }
}

impl AsRef<Http> for Context {
    fn as_ref(&self) -> &Http {
        &self.http
    }
}

impl AsRef<Http> for Arc<Context> {
    fn as_ref(&self) -> &Http {
        &self.http
    }
}

impl AsRef<Arc<Http>> for Context {
    fn as_ref(&self) -> &Arc<Http> {
        &self.http
    }
}

#[cfg(feature = "cache")]
impl AsRef<Cache> for Context {
    fn as_ref(&self) -> &Cache {
        &self.cache
    }
}

#[cfg(feature = "cache")]
impl AsRef<Cache> for Arc<Context> {
    fn as_ref(&self) -> &Cache {
        &self.cache
    }
}

#[cfg(feature = "cache")]
impl AsRef<Arc<Cache>> for Context {
    fn as_ref(&self) -> &Arc<Cache> {
        &self.cache
    }
}

#[cfg(feature = "cache")]
impl AsRef<Cache> for Cache {
    fn as_ref(&self) -> &Cache {
        self
    }
}

#[cfg(feature = "gateway")]
impl AsRef<ShardMessenger> for Context {
    fn as_ref(&self) -> &ShardMessenger {
        &self.shard
    }
}
