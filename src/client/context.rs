use crate::client::bridge::gateway::ShardMessenger;
use crate::gateway::InterMessage;
use crate::model::prelude::*;
use parking_lot::RwLock;
use std::sync::{
    Arc,
    mpsc::Sender,
};
use typemap::ShareMap;

use crate::http::Http;

#[cfg(feature = "cache")]
pub use crate::cache::{Cache, CacheRwLock};

/// The context is a general utility struct provided on event dispatches, which
/// helps with dealing with the current "context" of the event dispatch.
/// The context also acts as a general high-level interface over the associated
/// [`Shard`] which received the event, or the low-level [`http`] module.
///
/// The context contains "shortcuts", like for interacting with the shard.
/// Methods like [`set_activity`] will unlock the shard and perform an update for
/// you to save a bit of work.
///
/// A context will only live for the event it was dispatched for. After the
/// event handler finished, it is destroyed and will not be re-used.
///
/// [`Shard`]: ../gateway/struct.Shard.html
/// [`http`]: ../http/index.html
/// [`set_activity`]: #method.set_activity
#[derive(Clone)]
pub struct Context {
    /// A clone of [`Client::data`]. Refer to its documentation for more
    /// information.
    ///
    /// [`Client::data`]: struct.Client.html#structfield.data
    pub data: Arc<RwLock<ShareMap>>,
    /// The messenger to communicate with the shard runner.
    pub shard: ShardMessenger,
    /// The ID of the shard this context is related to.
    pub shard_id: u64,
    pub http: Arc<Http>,
    #[cfg(feature = "cache")]
    pub cache: CacheRwLock,
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    #[cfg(feature = "cache")]
    pub(crate) fn new(
        data: Arc<RwLock<ShareMap>>,
        runner_tx: Sender<InterMessage>,
        shard_id: u64,
        http: Arc<Http>,
        cache: Arc<RwLock<Cache>>,
    ) -> Context {
        Context {
            shard: ShardMessenger::new(runner_tx),
            shard_id,
            data,
            http,
            cache: cache.into(),
        }
    }

    /// Create a new Context to be passed to an event handler.
    #[cfg(not(feature = "cache"))]
    pub(crate) fn new(
        data: Arc<RwLock<ShareMap>>,
        runner_tx: Sender<InterMessage>,
        shard_id: u64,
        http: Arc<Http>,
    ) -> Context {
        Context {
            shard: ShardMessenger::new(runner_tx),
            shard_id,
            data,
            http,
        }
    }

    /// Sets the current user as being [`Online`]. This maintains the current
    /// activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being online on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!online" {
    ///             ctx.online();
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// [`Online`]: ../model/user/enum.OnlineStatus.html#variant.Online
    #[inline]
    pub fn online(&self) {
        self.shard.set_status(OnlineStatus::Online);
    }

    /// Sets the current user as being [`Idle`]. This maintains the current
    /// activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being idle on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!idle" {
    ///             ctx.idle();
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// [`Idle`]: ../model/user/enum.OnlineStatus.html#variant.Idle
    #[inline]
    pub fn idle(&self) {
        self.shard.set_status(OnlineStatus::Idle);
    }

    /// Sets the current user as being [`DoNotDisturb`]. This maintains the
    /// current activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being Do Not Disturb on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!dnd" {
    ///             ctx.dnd();
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/user/enum.OnlineStatus.html#variant.DoNotDisturb
    #[inline]
    pub fn dnd(&self) {
        self.shard.set_status(OnlineStatus::DoNotDisturb);
    }

    /// Sets the current user as being [`Invisible`]. This maintains the current
    /// activity.
    ///
    /// # Examples
    ///
    /// Set the current user to being invisible on the shard when an
    /// [`Event::Ready`] is received:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn ready(&self, ctx: Context, _: Ready) {
    ///         ctx.invisible();
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`Invisible`]: ../model/user/enum.OnlineStatus.html#variant.Invisible
    #[inline]
    pub fn invisible(&self) {
        self.shard.set_status(OnlineStatus::Invisible);
    }

    /// "Resets" the current user's presence, by setting the activity to `None`
    /// and the online status to [`Online`].
    ///
    /// Use [`set_presence`] for fine-grained control over individual details.
    ///
    /// # Examples
    ///
    /// Reset the presence when an [`Event::Resumed`] is received:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::event::ResumedEvent;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn resume(&self, ctx: Context, _: ResumedEvent) {
    ///         ctx.reset_presence();
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// [`Event::Resumed`]: ../model/event/enum.Event.html#variant.Resumed
    /// [`Online`]: ../model/user/enum.OnlineStatus.html#variant.Online
    /// [`set_presence`]: #method.set_presence
    #[inline]
    pub fn reset_presence(&self) {
        self.shard.set_presence(None::<Activity>, OnlineStatus::Online);
    }

    /// Sets the current activity, defaulting to an online status of [`Online`].
    ///
    /// # Examples
    ///
    /// Create a command named `~setgame` that accepts a name of a game to be
    /// playing:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "model")]
    /// # fn main() {
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// use serenity::model::gateway::Activity;
    ///
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, ctx: Context, msg: Message) {
    ///         let args = msg.content.splitn(2, ' ').collect::<Vec<&str>>();
    ///
    ///         if args.len() < 2 || *unsafe { args.get_unchecked(0) } != "~setgame" {
    ///             return;
    ///         }
    ///
    ///         ctx.set_activity(Activity::playing(*unsafe { args.get_unchecked(1) }));
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// # }
    ///
    /// # #[cfg(not(feature = "model"))]
    /// # fn main() {}
    /// ```
    ///
    /// [`Online`]: ../model/user/enum.OnlineStatus.html#variant.Online
    #[inline]
    pub fn set_activity(&self, activity: Activity) {
        self.shard.set_presence(Some(activity), OnlineStatus::Online);
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
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn ready(&self, ctx: Context, _: Ready) {
    ///         use serenity::model::user::OnlineStatus;
    ///
    ///         ctx.set_presence(None, OnlineStatus::Idle);
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// Setting the current user as playing `"Heroes of the Storm"`, while being
    /// [`DoNotDisturb`]:
    ///
    /// ```rust,ignore
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn ready(&self, context: Context, _: Ready) {
    ///         use serenity::model::gateway::Activity;
    ///         use serenity::model::user::OnlineStatus;
    ///
    ///         let activity = Activity::playing("Heroes of the Storm");
    ///         let status = OnlineStatus::DoNotDisturb;
    ///
    ///         context.set_presence(Some(activity), status);
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/user/enum.OnlineStatus.html#variant.DoNotDisturb
    /// [`Idle`]: ../model/user/enum.OnlineStatus.html#variant.Idle
    #[inline]
    pub fn set_presence(&self, activity: Option<Activity>, status: OnlineStatus) {
        self.shard.set_presence(activity, status);
    }
}

impl AsRef<Http> for Context {
    fn as_ref(&self) -> &Http { &self.http }
}

#[cfg(feature = "cache")]
impl AsRef<CacheRwLock> for Context {
    fn as_ref(&self) -> &CacheRwLock {
        &self.cache
    }
}
