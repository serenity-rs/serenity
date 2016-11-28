use super::utils::*;
use super::*;
use ::internal::prelude::*;

impl Game {
    /// Creates a `Game` struct that appears as a `**Playing** <name>` status.
    #[cfg(feature="methods")]
    pub fn playing(name: &str) -> Game {
        Game {
            kind: GameType::Playing,
            name: name.to_owned(),
            url: None,
        }
    }

    /// Creates a `Game` struct that appears as a `**Streaming** <name>` status.
    #[cfg(feature="methods")]
    pub fn streaming(name: &str, url: &str) -> Game {
        Game {
            kind: GameType::Streaming,
            name: name.to_owned(),
            url: Some(url.to_owned()),
        }
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Option<Game>> {
        let mut map = try!(into_map(value));

        let name = match map.remove("name") {
            Some(Value::Null) | None => return Ok(None),
            Some(v) => try!(into_string(v)),
        };

        if name.trim().is_empty() {
            return Ok(None);
        }

        missing!(map, Some(Game {
            name: name,
            kind: try!(opt(&mut map, "type", GameType::decode)).unwrap_or(GameType::Playing),
            url: try!(opt(&mut map, "url", into_string)),
        }))
    }
}

impl Presence {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Presence> {
        let mut value = try!(into_map(value));
        let mut user_map = try!(remove(&mut value, "user").and_then(into_map));

        let (user_id, user) = if user_map.len() > 1 {
            let user = try!(User::decode(Value::Object(user_map)));
            (user.id, Some(user))
        } else {
            (try!(remove(&mut user_map, "id").and_then(UserId::decode)), None)
        };

        let game = match value.remove("game") {
            None | Some(Value::Null) => None,
            Some(v) => try!(Game::decode(v)),
        };

        missing!(value, Presence {
            user_id: user_id,
            status: try!(remove(&mut value, "status").and_then(OnlineStatus::decode_str)),
            last_modified: try!(opt(&mut value, "last_modified", |v| Ok(req!(v.as_u64())))),
            game: game,
            user: user,
            nick: try!(opt(&mut value, "nick", into_string)),
        })
    }
}
