use std::sync::{Arc, Mutex};
use typemap::ShareMap;
use ::gateway::Shard;
use ::http;
use ::internal::prelude::*;
use ::model::*;

#[cfg(feature="cache")]
use super::CACHE;
#[cfg(feature="builder")]
use ::builder::EditProfile;

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
    /// The Id of the relevant channel, if there is one. This is present on the
    /// [`on_message`] handler, for example.
    ///
    /// [`on_message`]: struct.Client.html#method.on_message
    pub channel_id: Option<ChannelId>,
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
    /// The queue of messages that are sent after context goes out of scope.
    pub queue: String,
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    ///
    /// There's no real reason to use this yourself. But the option is there.
    /// Highly re-consider _not_ using this if you're tempted.
    ///
    /// Or don't do what I say. I'm just a comment hidden from the generated
    /// documentation.
    #[doc(hidden)]
    pub fn new(channel_id: Option<ChannelId>,
               shard: Arc<Mutex<Shard>>,
               data: Arc<Mutex<ShareMap>>) -> Context {
        Context {
            channel_id: channel_id,
            data: data,
            shard: shard,
            queue: String::new(),
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
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// #
    /// # client.on_message(|ctx, msg| {
    /// #     if msg.content == "!changename" {
    /// ctx.edit_profile(|p| p.username("Hakase"));
    /// #     }
    /// # });
    /// ```
    #[cfg(feature="builder")]
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
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// client.on_message(|ctx, msg| {
    ///     if msg.content == "!online" {
    ///         ctx.online();
    ///     }
    /// });
    /// ```
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    pub fn online(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Online);
    }

    /// Sets the current user as being [`Idle`]. This maintains the current
    /// game.
    ///
    /// # Examples
    ///
    /// Set the current user to being idle on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// client.on_message(|ctx, msg| {
    ///     if msg.content == "!idle" {
    ///         ctx.idle();
    ///     }
    /// });
    /// ```
    ///
    /// [`Idle`]: ../model/enum.OnlineStatus.html#variant.Idle
    pub fn idle(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Idle);
    }

    /// Sets the current user as being [`DoNotDisturb`]. This maintains the
    /// current game.
    ///
    /// # Examples
    ///
    /// Set the current user to being Do Not Disturb on the shard:
    ///
    /// ```rust,no_run
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// client.on_message(|ctx, msg| {
    ///     if msg.content == "!dnd" {
    ///         ctx.dnd();
    ///     }
    /// });
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/enum.OnlineStatus.html#variant.DoNotDisturb
    pub fn dnd(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::DoNotDisturb);
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
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// client.on_ready(|ctx, _| {
    ///     ctx.invisible();
    /// });
    /// ```
    ///
    /// [`Event::Ready`]: ../model/event/enum.Event.html#variant.Ready
    /// [`Invisible`]: ../model/enum.OnlineStatus.html#variant.Invisible
    pub fn invisible(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Invisible);
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
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// client.on_resume(|ctx, _| {
    ///     ctx.reset_presence();
    /// });
    /// ```
    ///
    /// [`Event::Resumed`]: ../model/event/enum.Event.html#variant.Resumed
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    /// [`set_presence`]: #method.set_presence
    pub fn reset_presence(&self) {
        self.shard.lock()
            .unwrap()
            .set_presence(None, OnlineStatus::Online, false)
    }

    /// Sets the current game, defaulting to an online status of [`Online`].
    ///
    /// # Examples
    ///
    /// Create a command named `~setgame` that accepts a name of a game to be
    /// playing:
    ///
    /// ```rust,no_run
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// #
    /// use serenity::model::Game;
    ///
    /// client.on_message(|ctx, msg| {
    ///     let args = msg.content.splitn(2, ' ').collect::<Vec<&str>>();
    ///
    ///     if args.len() < 2 || *unsafe { args.get_unchecked(0) } != "~setgame" {
    ///         return;
    ///     }
    ///
    ///     ctx.set_game(Game::playing(*unsafe { args.get_unchecked(1) }));
    /// });
    /// ```
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    pub fn set_game(&self, game: Game) {
        self.shard.lock()
            .unwrap()
            .set_presence(Some(game), OnlineStatus::Online, false);
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
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// #
    /// client.on_ready(|ctx, _| {
    ///     ctx.set_game_name("test");
    /// });
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

        self.shard.lock()
            .unwrap()
            .set_presence(Some(game), OnlineStatus::Online, false);
    }

    /// Sets the current user's presence, providing all fields to be passed.
    ///
    /// # Examples
    ///
    /// Setting the current user as having no game and being [`Idle`]:
    ///
    /// ```rust,no_run
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// #
    /// # client.on_ready(|ctx, _| {
    /// #
    /// use serenity::model::OnlineStatus;
    ///
    /// ctx.set_presence(None, OnlineStatus::Idle, false);
    /// # });
    /// ```
    ///
    /// Setting the current user as playing `"Heroes of the Storm"`, while being
    /// [`DoNotDisturb`]:
    ///
    /// ```rust,ignore
    /// # use serenity::Client;
    /// #
    /// # let mut client = Client::login("");
    /// #
    /// # client.on_ready(|ctx, _| {
    /// #
    /// use serenity::model::{Game, OnlineStatus};
    ///
    /// let game = Game::playing("Heroes of the Storm");
    /// let status = OnlineStatus::DoNotDisturb;
    ///
    /// context.set_presence(Some(game), status, false);
    /// # });
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/enum.OnlineStatus.html#variant.DoNotDisturb
    /// [`Idle`]: ../model/enum.OnlineStatus.html#variant.Idle
    pub fn set_presence(&self,
                        game: Option<Game>,
                        status: OnlineStatus,
                        afk: bool) {
        self.shard.lock()
            .unwrap()
            .set_presence(game, status, afk)
    }
}
