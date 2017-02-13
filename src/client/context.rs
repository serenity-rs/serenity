use serde_json::builder::ObjectBuilder;
use std::sync::{Arc, Mutex};
use super::gateway::Shard;
use super::rest::{self, GuildPagination};
use super::login_type::LoginType;
use typemap::ShareMap;
use ::utils::builder::EditProfile;
use ::internal::prelude::*;
use ::model::*;

#[cfg(feature="cache")]
use super::CACHE;

/// The context is a general utility struct provided on event dispatches, which
/// helps with dealing with the current "context" of the event dispatch.
/// The context also acts as a general high-level interface over the associated
/// [`Shard`] which received the event, or the low-level [`rest`] module.
///
/// The context contains "shortcuts", like for interacting with the shard.
/// Methods like [`set_game`] will unlock the shard and perform an update for
/// you to save a bit of work.
///
/// A context will only live for the event it was dispatched for. After the
/// event handler finished, it is destroyed and will not be re-used.
///
/// [`Shard`]: gateway/struct.Shard.html
/// [`rest`]: rest/index.html
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
    login_type: LoginType,
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
               data: Arc<Mutex<ShareMap>>,
               login_type: LoginType) -> Context {
        Context {
            channel_id: channel_id,
            data: data,
            shard: shard,
            login_type: login_type,
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
    /// ```rust,ignore
    /// context.edit_profile(|p| p.username("Hakase"));
    /// ```
    pub fn edit_profile<F: FnOnce(EditProfile) -> EditProfile>(&self, f: F) -> Result<CurrentUser> {
        let mut map = ObjectBuilder::new();

        feature_cache! {{
            let cache = CACHE.read().unwrap();

            map = map.insert("avatar", &cache.user.avatar)
                .insert("username", &cache.user.name);

            if let Some(email) = cache.user.email.as_ref() {
                map = map.insert("email", email);
            }
        } else {
            let user = rest::get_current_user()?;

            map = map.insert("avatar", user.avatar)
                .insert("username", user.name);

            if let Some(email) = user.email.as_ref() {
                map = map.insert("email", email);
            }
        }}

        let edited = f(EditProfile(map)).0.build();

        rest::edit_profile(edited)
    }

    /// Sets the current user as being [`Online`]. This maintains the current
    /// game and `afk` setting.
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    pub fn online(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Online);
    }

    /// Sets the current user as being [`Idle`]. This maintains the current
    /// game and `afk` setting.
    ///
    /// [`Idle`]: ../model/enum.OnlineStatus.html#variant.Idle
    pub fn idle(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Idle);
    }

    /// Sets the current user as being [`DoNotDisturb`]. This maintains the
    /// current game and `afk` setting.
    ///
    /// [`DoNotDisturb`]: ../model/enum.OnlineStatus.html#variant.DoNotDisturb
    pub fn dnd(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::DoNotDisturb);
    }

    /// Sets the current user as being [`Invisible`]. This maintains the current
    /// game and `afk` setting.
    ///
    /// [`Invisible`]: ../model/enum.OnlineStatus.html#variant.Invisible
    pub fn invisible(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Invisible);
    }

    /// "Resets" the current user's presence, by setting the game to `None`,
    /// the online status to [`Online`], and `afk` to `false`.
    ///
    /// Use [`set_presence`] for fine-grained control over individual details.
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    /// [`set_presence`]: #method.set_presence
    pub fn reset_presence(&self) {
        self.shard.lock()
            .unwrap()
            .set_presence(None, OnlineStatus::Online, false)
    }

    /// Sets the current game, defaulting to an online status of [`Online`], and
    /// setting `afk` to `false`.
    ///
    /// # Examples
    ///
    /// Set the current user as playing "Heroes of the Storm":
    ///
    /// ```rust,ignore
    /// use serenity::model::Game;
    ///
    /// // assuming you are in a context
    ///
    /// context.set_game(Game::playing("Heroes of the Storm"));
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
    /// Setting the current user as having no game, being [`Idle`],
    /// and setting `afk` to `true`:
    ///
    /// ```rust,ignore
    /// use serenity::model::OnlineStatus;
    ///
    /// // assuming you are in a context
    ///
    /// context.set_game(None, OnlineStatus::Idle, true);
    /// ```
    ///
    /// Setting the current user as playing "Heroes of the Storm", being
    /// [`DoNotDisturb`], and setting `afk` to `false`:
    ///
    /// ```rust,ignore
    /// use serenity::model::{Game, OnlineStatus};
    ///
    /// // assuming you are in a context
    ///
    /// let game = Game::playing("Heroes of the Storm");
    /// let status = OnlineStatus::DoNotDisturb;
    ///
    /// context.set_game(Some(game), status, false);
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
