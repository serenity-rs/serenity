use client::bridge::gateway::ShardMessenger;
use gateway::InterMessage;
use model::prelude::*;
use parking_lot::Mutex;
use std::sync::{
    mpsc::Sender,
    Arc
};
use typemap::ShareMap;

#[cfg(feature = "builder")]
use builder::EditProfile;
#[cfg(feature = "builder")]
use internal::prelude::*;
#[cfg(all(feature = "builder", feature = "cache"))]
use super::CACHE;
#[cfg(feature = "builder")]
use {Result, http};
#[cfg(feature = "builder")]
use utils::{self, VecMap};

/// The context is a general utility struct provided on event dispatches, which
/// helps with dealing with the current "context" of the event dispatch.
/// The context also acts as a general high-level interface over the associated
/// [`Shard`] which received the event, or the low-level [`http`] module.
///
/// The context contains "shortcuts", like for interacting with the shard.
/// Methods like [`set_game`] will unlock the shard and perform an update for
/// you to save a bit of work.
///
/// A context will only live for the event it was dispatched for. After the
/// event handler finished, it is destroyed and will not be re-used.
///
/// [`Shard`]: ../gateway/struct.Shard.html
/// [`http`]: ../http/index.html
/// [`set_game`]: #method.set_game
#[derive(Clone)]
pub struct Context {
    /// A clone of [`Client::data`]. Refer to its documentation for more
    /// information.
    ///
    /// [`Client::data`]: struct.Client.html#structfield.data
    pub data: Arc<Mutex<ShareMap>>,
    /// The messenger to communicate with the shard runner.
    pub shard: ShardMessenger,
    /// The ID of the shard this context is related to.
    pub shard_id: u64,
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    pub(crate) fn new(
        data: Arc<Mutex<ShareMap>>,
        runner_tx: Sender<InterMessage>,
        shard_id: u64,
    ) -> Context {
        Context {
            shard: ShardMessenger::new(runner_tx),
            shard_id,
            data,
        }
    }

    /// Edits the current user's profile settings.
    ///
    /// Refer to `EditProfile`'s documentation for its methods.
    ///
    /// # Examples
    ///
    /// Change the current user's username:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::channel::Message;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!changename" {
    ///             ctx.edit_profile(|e| e.username("Edward Elric"));
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    #[cfg(feature = "builder")]
    #[deprecated(since = "0.5.6", note = "Use the http module instead.")]
    pub fn edit_profile<F: FnOnce(EditProfile) -> EditProfile>(&self, f: F) -> Result<CurrentUser> {
        let mut map = VecMap::with_capacity(2);

        feature_cache! {
            {
                let cache = CACHE.read();

                map.insert("username", Value::String(cache.user.name.clone()));

                if let Some(email) = cache.user.email.clone() {
                    map.insert("email", Value::String(email));
                }
            } else {
                let user = http::get_current_user()?;

                map.insert("username", Value::String(user.name));

                if let Some(email) = user.email {
                    map.insert("email", Value::String(email));
                }
            }
        }

        let edited = utils::vecmap_to_json_map(f(EditProfile(map)).0);

        http::edit_profile(&edited)
    }


    /// Sets the current user as being [`Online`]. This maintains the current
    /// game.
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
    /// game.
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
    /// current game.
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
    /// game.
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

    /// "Resets" the current user's presence, by setting the game to `None` and
    /// the online status to [`Online`].
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
        self.shard.set_presence(None, OnlineStatus::Online);
    }

    /// Sets the current game, defaulting to an online status of [`Online`].
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
    /// use serenity::model::gateway::Game;
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
    ///         ctx.set_game(Game::playing(*unsafe { args.get_unchecked(1) }));
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
    pub fn set_game(&self, game: Game) {
        self.shard.set_presence(Some(game), OnlineStatus::Online);
    }

    /// Sets the current game, passing in only its name. This will automatically
    /// set the current user's [`OnlineStatus`] to [`Online`], and its
    /// [`GameType`] as [`Playing`].
    ///
    /// Use [`reset_presence`] to clear the current game, or [`set_presence`]
    /// for more fine-grained control.
    ///
    /// **Note**: Maximum length is 128.
    ///
    /// # Examples
    ///
    /// When an [`Event::Ready`] is received, set the game name to `"test"`:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn ready(&self, ctx: Context, _: Ready) {
    ///         ctx.set_game_name("test");
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    /// client.start().unwrap();
    /// ```
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`GameType`]: ../model/gateway/enum.GameType.html
    /// [`Online`]: ../model/user/enum.OnlineStatus.html#variant.Online
    /// [`OnlineStatus`]: ../model/user/enum.OnlineStatus.html
    /// [`Playing`]: ../model/gateway/enum.GameType.html#variant.Playing
    /// [`reset_presence`]: #method.reset_presence
    /// [`set_presence`]: #method.set_presence
    pub fn set_game_name(&self, game_name: &str) {
        let game = Game {
            kind: GameType::Playing,
            name: game_name.to_string(),
            url: None,
        };

        self.shard.set_presence(Some(game), OnlineStatus::Online);
    }

    /// Sets the current user's presence, providing all fields to be passed.
    ///
    /// # Examples
    ///
    /// Setting the current user as having no game and being [`Idle`]:
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
    ///         use serenity::model::gateway::Game;
    ///         use serenity::model::user::OnlineStatus;
    ///
    ///         let game = Game::playing("Heroes of the Storm");
    ///         let status = OnlineStatus::DoNotDisturb;
    ///
    ///         context.set_presence(Some(game), status);
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
    pub fn set_presence(&self, game: Option<Game>, status: OnlineStatus) {
        self.shard.set_presence(game, status);
    }

    /// Disconnects the shard from the websocket, essentially "quiting" it.
    /// Note however that this will only exit the one which the `Context` was given.
    /// If it's just one shard that's on, then serenity will stop any further actions
    /// until [`Client::start`] and vice versa are called again.
    ///
    /// [`Client::start`]: ./struct.Client.html#method.start
    #[inline]
    pub fn quit(&self) {
        self.shard.shutdown_clean();
    }
}
