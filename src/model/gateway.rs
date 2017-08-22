use serde::de::Error as DeError;
use serde_json;
use std::sync::{Arc, RwLock};
use super::utils::*;
use super::*;

/// A representation of the data retrieved from the bot gateway endpoint.
///
/// This is different from the [`Gateway`], as this includes the number of
/// shards that Discord recommends to use for a bot user.
///
/// This is only applicable to bot users.
#[derive(Clone, Debug, Deserialize)]
pub struct BotGateway {
    /// The number of shards that is recommended to be used by the current bot
    /// user.
    pub shards: u64,
    /// The gateway to connect to.
    pub url: String,
}

/// Representation of a game that a [`User`] is playing -- or streaming in the
/// case that a stream URL is provided.
#[derive(Clone, Debug)]
pub struct Game {
    /// The type of game status.
    pub kind: GameType,
    /// The name of the game being played.
    pub name: String,
    /// The Stream URL if [`kind`] is [`GameType::Streaming`].
    ///
    /// [`GameType::Streaming`]: enum.GameType.html#variant.Streaming
    /// [`kind`]: #structfield.kind
    pub url: Option<String>,
}

#[cfg(feature = "model")]
impl Game {
    /// Creates a `Game` struct that appears as a `Playing <name>` status.
    ///
    /// **Note**: Maximum `name` length is 128.
    ///
    /// # Examples
    ///
    /// Create a command that sets the current game being played:
    ///
    /// ```rust,no_run
    /// # #[macro_use] extern crate serenity;
    /// #
    /// use serenity::framework::standard::Args;
    /// use serenity::model::Game;
    ///
    /// command!(game(ctx, _msg, args) {
    ///     let name = args.join(" ");
    ///     ctx.set_game(Game::playing(&name));
    /// });
    /// #
    /// # fn main() {}
    /// ```
    pub fn playing(name: &str) -> Game {
        Game {
            kind: GameType::Playing,
            name: name.to_owned(),
            url: None,
        }
    }

    /// Creates a `Game` struct that appears as a `Streaming <name>` status.
    ///
    /// **Note**: Maximum `name` length is 128.
    ///
    /// # Examples
    ///
    /// Create a command that sets the current game and stream:
    ///
    /// ```rust,no_run
    /// # #[macro_use] extern crate serenity;
    /// #
    /// use serenity::framework::standard::Args;
    /// use serenity::model::Game;
    ///
    /// // Assumes command has min_args set to 2.
    /// command!(stream(ctx, _msg, args) {
    ///     # let stream_url = String::from("");
    ///     let name = args.full();
    ///     ctx.set_game(Game::streaming(&name, &stream_url));
    /// });
    /// #
    /// # fn main() {}
    /// ```
    pub fn streaming(name: &str, url: &str) -> Game {
        Game {
            kind: GameType::Streaming,
            name: name.to_owned(),
            url: Some(url.to_owned()),
        }
    }
}

impl<'de> Deserialize<'de> for Game {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;
        let kind = map.remove("type")
            .and_then(|v| GameType::deserialize(v).ok())
            .unwrap_or(GameType::Playing);
        let name = map.remove("name")
            .and_then(|v| String::deserialize(v).ok())
            .unwrap_or_else(String::new);
        let url = map.remove("url").and_then(|v| {
            serde_json::from_value::<String>(v).ok()
        });

        Ok(Game {
            kind: kind,
            name: name,
            url: url,
        })
    }
}

enum_number!(
    /// The type of activity that is being performed when playing a game.
    GameType {
        /// An indicator that the user is playing a game.
        Playing = 0,
        /// An indicator that the user is streaming to a service.
        Streaming = 1,
    }
);

impl Default for GameType {
    fn default() -> Self { GameType::Playing }
}

/// A representation of the data retrieved from the gateway endpoint.
///
/// For the bot-specific gateway, refer to [`BotGateway`].
///
/// [`BotGateway`]: struct.BotGateway.html
#[derive(Clone, Debug, Deserialize)]
pub struct Gateway {
    /// The gateway to connect to.
    pub url: String,
}

/// Information detailing the current online status of a [`User`].
///
/// [`User`]: struct.User.html
#[derive(Clone, Debug)]
pub struct Presence {
    /// The game that a [`User`] is current playing.
    ///
    /// [`User`]: struct.User.html
    pub game: Option<Game>,
    /// The date of the last presence update.
    pub last_modified: Option<u64>,
    /// The nickname of the member, if applicable.
    pub nick: Option<String>,
    /// The user's online status.
    pub status: OnlineStatus,
    /// The Id of the [`User`]. Can be used to calculate the user's creation
    /// date.
    pub user_id: UserId,
    /// The associated user instance.
    pub user: Option<Arc<RwLock<User>>>,
}

impl<'de> Deserialize<'de> for Presence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Presence, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;
        let mut user_map = map.remove("user")
            .ok_or_else(|| DeError::custom("expected presence user"))
            .and_then(JsonMap::deserialize)
            .map_err(DeError::custom)?;

        let (user_id, user) = if user_map.len() > 1 {
            let user = User::deserialize(Value::Object(user_map)).map_err(
                DeError::custom,
            )?;

            (user.id, Some(Arc::new(RwLock::new(user))))
        } else {
            let user_id = user_map
                .remove("id")
                .ok_or_else(|| DeError::custom("Missing presence user id"))
                .and_then(|x| UserId::deserialize(x.clone()))
                .map_err(DeError::custom)?;

            (user_id, None)
        };

        let game = match map.remove("game") {
            Some(v) => {
                serde_json::from_value::<Option<Game>>(v).map_err(
                    DeError::custom,
                )?
            },
            None => None,
        };
        let last_modified = match map.remove("last_modified") {
            Some(v) => Some(u64::deserialize(v).map_err(DeError::custom)?),
            None => None,
        };
        let nick = match map.remove("nick") {
            Some(v) => {
                serde_json::from_value::<Option<String>>(v).map_err(
                    DeError::custom,
                )?
            },
            None => None,
        };
        let status = map.remove("status")
            .ok_or_else(|| DeError::custom("expected presence status"))
            .and_then(OnlineStatus::deserialize)
            .map_err(DeError::custom)?;

        Ok(Presence {
            game: game,
            last_modified: last_modified,
            nick: nick,
            status: status,
            user: user,
            user_id: user_id,
        })
    }
}

/// An initial set of information given after IDENTIFYing to the gateway.
#[derive(Clone, Debug, Deserialize)]
pub struct Ready {
    pub guilds: Vec<GuildStatus>,
    #[serde(deserialize_with = "deserialize_presences")]
    pub presences: HashMap<UserId, Presence>,
    #[serde(deserialize_with = "deserialize_private_channels")]
    pub private_channels: HashMap<ChannelId, Channel>,
    pub session_id: String,
    pub shard: Option<[u64; 2]>,
    #[serde(default, rename = "_trace")]
    pub trace: Vec<String>,
    pub user: CurrentUser,
    #[serde(rename = "v")]
    pub version: u64,
}
