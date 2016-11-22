use std::collections::{BTreeMap, HashMap};
use super::{
    Channel,
    ChannelId,
    Emoji,
    EmojiId,
    Member,
    Presence,
    ReadState,
    Relationship,
    Role,
    RoleId,
    User,
    UserId,
    VoiceState,
};
use ::internal::prelude::*;
use ::utils::{decode_array, into_array};

#[cfg(feature = "methods")]
use super::permissions::{self, Permissions};
#[cfg(all(feature = "cache", feature = "methods"))]
use ::client::CACHE;

#[macro_escape]
macro_rules! missing {
    (@ $name:expr, $json:ident, $value:expr) => {
        (Ok($value), warn_field($name, $json)).0
    };
    ($json:ident, $ty:ident $(::$ext:ident)* ( $($value:expr),*$(,)* ) ) => {
        (Ok($ty$(::$ext)* ( $($value),* )), warn_field(stringify!($ty$(::$ext)*), $json)).0
    };
    ($json:ident, $ty:ident $(::$ext:ident)* { $($name:ident: $value:expr),*$(,)* } ) => {
        (Ok($ty$(::$ext)* { $($name: $value),* }), warn_field(stringify!($ty$(::$ext)*), $json)).0
    };
}

#[macro_escape]
macro_rules! req {
    ($opt:expr) => {
        try!($opt.ok_or(Error::Decode(concat!("Type mismatch in model:",
                                              line!(),
                                              ": ",
                                              stringify!($opt)),
                                      Value::Null)))
    }
}

pub fn decode_emojis(value: Value) -> Result<HashMap<EmojiId, Emoji>> {
    let mut emojis = HashMap::new();

    for emoji in try!(decode_array(value, Emoji::decode)) {
        emojis.insert(emoji.id, emoji);
    }

    Ok(emojis)
}

pub fn decode_experiments(value: Value) -> Result<Vec<Vec<u64>>> {
    let array = match value {
        Value::Array(v) => v,
        value => return Err(Error::Decode("Expected experiment array", value)),
    };

    let mut experiments: Vec<Vec<u64>> = vec![];

    for arr in array {
        let arr = match arr {
            Value::Array(v) => v,
            value => return Err(Error::Decode("Expected experiment's array", value)),
        };

        let mut items: Vec<u64> = vec![];

        for item in arr {
            items.push(match item {
                Value::I64(v) => v as u64,
                Value::U64(v) => v,
                value => return Err(Error::Decode("Expected experiment u64", value)),
            });
        }

        experiments.push(items);
    }

    Ok(experiments)
}

pub fn decode_id(value: Value) -> Result<u64> {
    match value {
        Value::U64(num) => Ok(num),
        Value::I64(num) => Ok(num as u64),
        Value::String(text) => match text.parse::<u64>() {
            Ok(num) => Ok(num),
            Err(_) => Err(Error::Decode("Expected numeric ID",
                                        Value::String(text)))
        },
        value => Err(Error::Decode("Expected numeric ID", value))
    }
}

pub fn decode_members(value: Value) -> Result<HashMap<UserId, Member>> {
    let mut members = HashMap::new();

    for member in try!(decode_array(value, Member::decode)) {
        members.insert(member.user.id, member);
    }

    Ok(members)
}

// Clippy's lint is incorrect here and will result in invalid code.
#[allow(or_fun_call)]
pub fn decode_notes(value: Value) -> Result<HashMap<UserId, String>> {
    let mut notes = HashMap::new();

    for (key, value) in into_map(value).unwrap_or(BTreeMap::default()) {
        let id = UserId(try!(key.parse::<u64>()
            .map_err(|_| Error::Decode("Invalid user id in notes",
                                       Value::String(key)))));

        notes.insert(id, try!(into_string(value)));
    }

    Ok(notes)
}

pub fn decode_presences(value: Value) -> Result<HashMap<UserId, Presence>> {
    let mut presences = HashMap::new();

    for presence in try!(decode_array(value, Presence::decode)) {
        presences.insert(presence.user_id, presence);
    }

    Ok(presences)
}

pub fn decode_private_channels(value: Value)
    -> Result<HashMap<ChannelId, Channel>> {
    let mut private_channels = HashMap::new();

    for private_channel in try!(decode_array(value, Channel::decode)) {
        let id = match private_channel {
            Channel::Group(ref group) => group.channel_id,
            Channel::Private(ref channel) => channel.id,
            Channel::Public(_) => unreachable!("Public private channel decode"),
        };

        private_channels.insert(id, private_channel);
    }

    Ok(private_channels)
}

pub fn decode_read_states(value: Value)
    -> Result<HashMap<ChannelId, ReadState>> {
    let mut read_states = HashMap::new();

    for read_state in try!(decode_array(value, ReadState::decode)) {
        read_states.insert(read_state.id, read_state);
    }

    Ok(read_states)
}

pub fn decode_relationships(value: Value)
    -> Result<HashMap<UserId, Relationship>> {
    let mut relationships = HashMap::new();

    for relationship in try!(decode_array(value, Relationship::decode)) {
        relationships.insert(relationship.id, relationship);
    }

    Ok(relationships)
}

pub fn decode_roles(value: Value) -> Result<HashMap<RoleId, Role>> {
    let mut roles = HashMap::new();

    for role in try!(decode_array(value, Role::decode)) {
        roles.insert(role.id, role);
    }

    Ok(roles)
}

pub fn decode_shards(value: Value) -> Result<[u8; 2]> {
    let array = try!(into_array(value));

    Ok([
        req!(try!(array.get(0)
            .ok_or(Error::Client(ClientError::InvalidShards))).as_u64()) as u8,
        req!(try!(array.get(1)
            .ok_or(Error::Client(ClientError::InvalidShards))).as_u64()) as u8,
    ])
}

pub fn decode_users(value: Value) -> Result<HashMap<UserId, User>> {
    let mut users = HashMap::new();

    for user in try!(decode_array(value, User::decode)) {
        users.insert(user.id, user);
    }

    Ok(users)
}

pub fn decode_voice_states(value: Value)
    -> Result<HashMap<UserId, VoiceState>> {
    let mut voice_states = HashMap::new();

    for voice_state in try!(decode_array(value, VoiceState::decode)) {
        voice_states.insert(voice_state.user_id, voice_state);
    }

    Ok(voice_states)
}

pub fn into_string(value: Value) -> Result<String> {
    match value {
        Value::String(s) => Ok(s),
        Value::U64(v) => Ok(v.to_string()),
        Value::I64(v) => Ok(v.to_string()),
        value => Err(Error::Decode("Expected string", value)),
    }
}

pub fn into_map(value: Value) -> Result<BTreeMap<String, Value>> {
    match value {
        Value::Object(m) => Ok(m),
        value => Err(Error::Decode("Expected object", value)),
    }
}

pub fn into_u64(value: Value) -> Result<u64> {
    match value {
        Value::I64(v) => Ok(v as u64),
        Value::String(v) => match v.parse::<u64>() {
            Ok(v) => Ok(v),
            Err(_why) => Err(Error::Decode("Expected valid u64", Value::String(v))),
        },
        Value::U64(v) => Ok(v),
        value => Err(Error::Decode("Expected u64", value)),
    }
}

pub fn opt<T, F: FnOnce(Value) -> Result<T>>(map: &mut BTreeMap<String, Value>, key: &str, f: F) -> Result<Option<T>> {
    match map.remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(val) => f(val).map(Some),
    }
}

pub fn parse_discriminator(value: Value) -> Result<u16> {
    match value {
        Value::I64(v) => Ok(v as u16),
        Value::U64(v) => Ok(v as u16),
        Value::String(s) => match s.parse::<u16>() {
            Ok(v) => Ok(v),
            Err(_why) => Err(Error::Decode("Error parsing discriminator as u16",
                                           Value::String(s))),
        },
        value => Err(Error::Decode("Expected string or u64", value)),
    }
}

pub fn remove(map: &mut BTreeMap<String, Value>, key: &str) -> Result<Value> {
    map.remove(key).ok_or_else(|| {
        Error::Decode("Unexpected absent key", Value::String(key.into()))
    })
}

#[doc(hidden)]
#[cfg(all(feature = "cache", feature="methods"))]
pub fn user_has_perms(channel_id: ChannelId,
                      mut permissions: Permissions)
                      -> Result<bool> {
    let cache = CACHE.lock().unwrap();
    let current_user = &cache.user;

    let channel = match cache.get_channel(channel_id) {
        Some(channel) => channel,
        None => return Err(Error::Client(ClientError::ItemMissing)),
    };

    let guild_id = match channel {
        Channel::Group(_) | Channel::Private(_) => {
            return Ok(permissions == permissions::MANAGE_MESSAGES);
        },
        Channel::Public(channel) => channel.guild_id,
    };

    let guild = match cache.get_guild(guild_id) {
        Some(guild) => guild,
        None => return Err(Error::Client(ClientError::ItemMissing)),
    };

    let perms = guild.permissions_for(channel_id, current_user.id);

    permissions.remove(perms);

    Ok(permissions.is_empty())
}

pub fn warn_field(name: &str, map: BTreeMap<String, Value>) {
    if !map.is_empty() {
        debug!("Unhandled keys: {} has {:?}", name, Value::Object(map))
    }
}
