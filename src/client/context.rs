use std::fmt;
use std::sync::Arc;

#[cfg(feature = "cache")]
pub use crate::cache::Cache;
use crate::gateway::ActivityData;
#[cfg(feature = "gateway")]
use crate::gateway::{ShardMessenger, ShardRunner};
use crate::http::Http;
use crate::model::prelude::*;

/// The context is a general utility struct provided on event dispatches.
///
/// The Context helps with dealing with the current "context" of the event dispatch. The context
/// also acts as a general high-level interface over the associated [`Shard`] which received
/// the event, or the low-level [`http`] module.
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
    data: Arc<dyn std::any::Any + Send + Sync>,
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
        data: Arc<dyn std::any::Any + Send + Sync>,
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
    pub fn easy(
        data: Arc<dyn std::any::Any + Send + Sync>,
        shard_id: ShardId,
        http: Arc<Http>,
    ) -> Context {
        Context {
            shard_id,
            data,
            http,
        }
    }

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
    pub fn set_presence(&self, activity: Option<ActivityData>, status: OnlineStatus) {
        self.shard.set_presence(activity, status);
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
    /// See [`Guild::create_emoji`] for information about name and filesize requirements. This
    /// method will error if said requirements are not met.
    pub async fn create_application_emoji(&self, name: &str, image: &str) -> Result<Emoji> {
        #[derive(serde::Serialize)]
        struct CreateEmoji<'a> {
            name: &'a str,
            image: &'a str,
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
