use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use super::{
    Channel,
    ChannelId,
    Emoji,
    EmojiId,
    Member,
    Message,
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

#[cfg(feature="cache")]
use super::permissions::{self, Permissions};
#[cfg(feature="cache")]
use ::client::CACHE;

#[macro_escape]
macro_rules! req {
    ($opt:expr) => {
        $opt.ok_or(Error::Decode(concat!("Type mismatch in model:",
                                         line!(),
                                         ": ",
                                         stringify!($opt)),
                                      Value::Null))?
    }
}

pub fn decode_emojis(value: Value) -> Result<HashMap<EmojiId, Emoji>> {
    let mut emojis = HashMap::new();

    for emoji in decode_array(value, Emoji::decode)? {
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

    for member in decode_array(value, Member::decode)? {
        let user_id = member.user.read().unwrap().id;

        members.insert(user_id, member);
    }

    Ok(members)
}

// Clippy's lint is incorrect here and will result in invalid code.
//
// Bit more detaul: `result_unwrap_or_default` is not yet stable as of rustc
// 1.14.
#[allow(or_fun_call)]
pub fn decode_notes(value: Value) -> Result<HashMap<UserId, String>> {
    let mut notes = HashMap::new();

    for (key, value) in into_map(value).unwrap_or(BTreeMap::default()) {
        let id = UserId(key.parse::<u64>()
            .map_err(|_| Error::Decode("Invalid user id in notes",
                                       Value::String(key)))?);

        notes.insert(id, into_string(value)?);
    }

    Ok(notes)
}

pub fn decode_presences(value: Value) -> Result<HashMap<UserId, Presence>> {
    let mut presences = HashMap::new();

    for presence in decode_array(value, Presence::decode)? {
        presences.insert(presence.user_id, presence);
    }

    Ok(presences)
}

pub fn decode_private_channels(value: Value)
    -> Result<HashMap<ChannelId, Channel>> {
    let mut private_channels = HashMap::new();

    for private_channel in decode_array(value, Channel::decode)? {
        let id = match private_channel {
            Channel::Group(ref group) => group.read().unwrap().channel_id,
            Channel::Private(ref channel) => channel.read().unwrap().id,
            Channel::Guild(_) => unreachable!("Guild private channel decode"),
        };

        private_channels.insert(id, private_channel);
    }

    Ok(private_channels)
}

pub fn decode_read_states(value: Value)
    -> Result<HashMap<ChannelId, ReadState>> {
    let mut read_states = HashMap::new();

    for read_state in decode_array(value, ReadState::decode)? {
        read_states.insert(read_state.id, read_state);
    }

    Ok(read_states)
}

pub fn decode_relationships(value: Value)
    -> Result<HashMap<UserId, Relationship>> {
    let mut relationships = HashMap::new();

    for relationship in decode_array(value, Relationship::decode)? {
        relationships.insert(relationship.id, relationship);
    }

    Ok(relationships)
}

pub fn decode_roles(value: Value) -> Result<HashMap<RoleId, Role>> {
    let mut roles = HashMap::new();

    for role in decode_array(value, Role::decode)? {
        roles.insert(role.id, role);
    }

    Ok(roles)
}

pub fn decode_search_results(value: Value) -> Result<Vec<Vec<Message>>> {
    let array = match value {
        Value::Array(v) => v,
        value => return Err(Error::Decode("Expected message set array", value)),
    };

    let mut sets: Vec<Vec<Message>> = vec![];

    for arr in array {
        let arr = match arr {
            Value::Array(v) => v,
            value => return Err(Error::Decode("Expected message set array", value)),
        };

        let mut messages: Vec<Message> = vec![];

        for item in arr {
            messages.push(match item {
                Value::Object(v) => try!(Message::decode(Value::Object(v))),
                value => return Err(Error::Decode("Expected search message", value)),
            });
        }

        sets.push(messages);
    }

    Ok(sets)
}

pub fn decode_shards(value: Value) -> Result<[u64; 2]> {
    let array = into_array(value)?;

    Ok([
        req!(array.get(0)
            .ok_or(Error::Client(ClientError::InvalidShards))?.as_u64()) as u64,
        req!(array.get(1)
            .ok_or(Error::Client(ClientError::InvalidShards))?.as_u64()) as u64,
    ])
}

pub fn decode_users(value: Value) -> Result<HashMap<UserId, Arc<RwLock<User>>>> {
    let mut users = HashMap::new();

    for user in decode_array(value, User::decode)? {
        users.insert(user.id, Arc::new(RwLock::new(user)));
    }

    Ok(users)
}

pub fn decode_voice_states(value: Value)
    -> Result<HashMap<UserId, VoiceState>> {
    let mut voice_states = HashMap::new();

    for voice_state in decode_array(value, VoiceState::decode)? {
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
            Err(_) => Err(Error::Decode("Expected valid u64", Value::String(v))),
        },
        Value::U64(v) => Ok(v),
        value => Err(Error::Decode("Expected u64", value)),
    }
}

pub fn opt<F, T>(map: &mut BTreeMap<String, Value>, key: &str, f: F)
    -> Result<Option<T>> where F: FnOnce(Value) -> Result<T> {
    match map.remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(val) => f(val).map(Some),
    }
}

pub fn decode_discriminator(value: Value) -> Result<u16> {
    match value {
        Value::I64(v) => Ok(v as u16),
        Value::U64(v) => Ok(v as u16),
        Value::String(s) => match s.parse::<u16>() {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::Decode("Error parsing discriminator as u16",
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

#[cfg(feature="cache")]
pub fn user_has_perms(channel_id: ChannelId,
                      mut permissions: Permissions)
                      -> Result<bool> {
    let cache = CACHE.read().unwrap();
    let current_user = &cache.user;

    let channel = match cache.get_channel(channel_id) {
        Some(channel) => channel,
        None => return Err(Error::Client(ClientError::ItemMissing)),
    };

    let guild_id = match channel {
        Channel::Group(_) | Channel::Private(_) => {
            return Ok(permissions == permissions::MANAGE_MESSAGES);
        },
        Channel::Guild(channel) => channel.read().unwrap().guild_id,
    };

    let guild = match cache.get_guild(guild_id) {
        Some(guild) => guild,
        None => return Err(Error::Client(ClientError::ItemMissing)),
    };

    let perms = guild.read().unwrap().permissions_for(channel_id, current_user.id);

    permissions.remove(perms);

    Ok(permissions.is_empty())
}
