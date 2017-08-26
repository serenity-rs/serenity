use std::sync::Arc;
use typemap::ShareMap;
use gateway::Shard;
use model::*;
use parking_lot::Mutex;

#[cfg(feature = "cache")]
use super::CACHE;
#[cfg(feature = "builder")]
use internal::prelude::*;
#[cfg(feature = "builder")]
use builder::EditProfile;
#[cfg(feature = "builder")]
use http;

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
    /// The associated shard which dispatched the event handler.
    ///
    /// Note that if you are sharding, in relevant terms, this is the shard
    /// which received the event being dispatched.
    pub shard: Arc<Mutex<Shard>>,
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    pub(crate) fn new(shard: Arc<Mutex<Shard>>, data: Arc<Mutex<ShareMap>>) -> Context {
        Context {
            data,
            shard,
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
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!changename" {
    ///             ctx.edit_profile(|e| e.username("Edward Elric"));
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    #[cfg(feature = "builder")]
    pub fn edit_profile<F: FnOnce(EditProfile) -> EditProfile>(&self, f: F) -> Result<CurrentUser> {
        let mut map = Map::new();

        feature_cache! {{
                            let cache = CACHE.read().unwrap();
        
                            map.insert("username".to_owned(), Value::String(cache.user.name.clone()));
        
                            if let Some(email) = cache.user.email.as_ref() {
                                map.insert("email".to_owned(), Value::String(email.clone()));
                            }
                        } else {
                            let user = http::get_current_user()?;
        
                            map.insert("username".to_owned(), Value::String(user.name.clone()));
        
                            if let Some(email) = user.email.as_ref() {
                                map.insert("email".to_owned(), Value::String(email.clone()));
                            }
                        }}

        let edited = f(EditProfile(map)).0;

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
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!online" {
    ///             ctx.online();
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    pub fn online(&self) {
        let mut shard = self.shard.lock();
        shard.set_status(OnlineStatus::Online);
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
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!idle" {
    ///             ctx.idle();
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Idle`]: ../model/enum.OnlineStatus.html#variant.Idle
    pub fn idle(&self) {
        let mut shard = self.shard.lock();
        shard.set_status(OnlineStatus::Idle);
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
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "!dnd" {
    ///             ctx.dnd();
    ///         }
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/enum.OnlineStatus.html#variant.DoNotDisturb
    pub fn dnd(&self) {
        let mut shard = self.shard.lock();
        shard.set_status(OnlineStatus::DoNotDisturb);
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
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_ready(&self, ctx: Context, _: Ready) {
    ///         ctx.invisible();
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`Invisible`]: ../model/enum.OnlineStatus.html#variant.Invisible
    pub fn invisible(&self) {
        let mut shard = self.shard.lock();
        shard.set_status(OnlineStatus::Invisible);
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
    ///     fn on_resume(&self, ctx: Context, _: ResumedEvent) {
    ///         ctx.reset_presence();
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Event::Resumed`]: ../model/event/enum.Event.html#variant.Resumed
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    /// [`set_presence`]: #method.set_presence
    pub fn reset_presence(&self) {
        let mut shard = self.shard.lock();
        shard.set_presence(None, OnlineStatus::Online, false)
    }

    /// Sets the current game, defaulting to an online status of [`Online`].
    ///
    /// # Examples
    ///
    /// Create a command named `~setgame` that accepts a name of a game to be
    /// playing:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::*;
    /// #
    /// use serenity::model::Game;
    ///
    /// struct Handler;
    /// impl EventHandler for Handler {
    ///     fn on_message(&self, ctx: Context, msg: Message) {
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
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    pub fn set_game(&self, game: Game) {
        let mut shard = self.shard.lock();
        shard.set_presence(Some(game), OnlineStatus::Online, false);
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
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_ready(&self, ctx: Context, _: Ready) {
    ///         ctx.set_game_name("test");
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`GameType`]: ../model/enum.GameType.html
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    /// [`OnlineStatus`]: ../model/enum.OnlineStatus.html
    /// [`Playing`]: ../model/enum.GameType.html#variant.Playing
    /// [`reset_presence`]: #method.reset_presence
    /// [`set_presence`]: #method.set_presence
    pub fn set_game_name(&self, game_name: &str) {
        let game = Game {
            kind: GameType::Playing,
            name: game_name.to_owned(),
            url: None,
        };

        let mut shard = self.shard.lock();
        shard.set_presence(Some(game), OnlineStatus::Online, false);
    }

    /// Sets the current user's presence, providing all fields to be passed.
    ///
    /// # Examples
    ///
    /// Setting the current user as having no game and being [`Idle`]:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_ready(&self, ctx: Context, _: Ready) {
    ///         use serenity::model::OnlineStatus;
    ///
    ///         ctx.set_presence(None, OnlineStatus::Idle, false);
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// Setting the current user as playing `"Heroes of the Storm"`, while being
    /// [`DoNotDisturb`]:
    ///
    /// ```rust,ignore
    /// # use serenity::prelude::*;
    /// # use serenity::model::*;
    /// #
    /// struct Handler;
    ///
    /// impl EventHandler for Handler {
    ///     fn on_ready(&self, context: Context, _: Ready) {
    ///         use serenity::model::{Game, OnlineStatus};
    ///
    ///         let game = Game::playing("Heroes of the Storm");
    ///         let status = OnlineStatus::DoNotDisturb;
    ///
    ///         context.set_presence(Some(game), status, false);
    ///     }
    /// }
    /// let mut client = Client::new("token", Handler); client.start().unwrap();
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/enum.OnlineStatus.html#variant.DoNotDisturb
    /// [`Idle`]: ../model/enum.OnlineStatus.html#variant.Idle
    pub fn set_presence(&self, game: Option<Game>, status: OnlineStatus, afk: bool) {
        let mut shard = self.shard.lock();
        shard.set_presence(game, status, afk)
    }

    /// Disconnects the shard from the websocket, essentially "quiting" it.
    /// Note however that this will only exit the one which the `Context` was given.
    /// If it's just one shard that's on, then serenity will stop any further actions
    /// until [`Client::start`] and vice versa are called again.
    ///
    /// [`Client::start`]: ./struct.Client.html#method.start
    pub fn quit(&self) -> Result<()> {
        let mut shard = self.shard.lock();
        shard.shutdown_clean()
    }
}
