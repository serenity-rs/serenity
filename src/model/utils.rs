use serde::de::Error as DeError;
use serde::ser::{SerializeSeq, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use super::prelude::*;

#[cfg(all(feature = "cache", feature = "model"))]
use cache::Cache;

#[cfg(feature = "cache")]
use internal::prelude::*;

#[cfg(all(feature = "cache", feature = "model"))]
use super::permissions::Permissions;

pub fn default_true() -> bool {
    true
}

pub fn deserialize_emojis<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<EmojiId, Emoji>, D::Error> {
    let vec: Vec<Emoji> = Deserialize::deserialize(deserializer)?;
    let mut emojis = HashMap::new();

    for emoji in vec {
        emojis.insert(emoji.id, emoji);
    }

    Ok(emojis)
}

pub fn deserialize_guild_channels<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<ChannelId, Rc<RefCell<GuildChannel>>>, D::Error> {
    let vec: Vec<GuildChannel> = Deserialize::deserialize(deserializer)?;
    let mut map = HashMap::new();

    for channel in vec {
        map.insert(channel.id, Rc::new(RefCell::new(channel)));
    }

    Ok(map)
}

pub fn deserialize_members<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<UserId, Rc<RefCell<Member>>>, D::Error> {
    let vec: Vec<Member> = Deserialize::deserialize(deserializer)?;
    let mut members = HashMap::new();

    for member in vec {
        let user_id = member.user.borrow().id;

        members.insert(user_id, Rc::new(RefCell::new(member)));
    }

    Ok(members)
}

pub fn deserialize_presences<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<UserId, Rc<RefCell<Presence>>>, D::Error> {
    let vec: Vec<Presence> = Deserialize::deserialize(deserializer)?;
    let mut presences = HashMap::new();

    for presence in vec {
        presences.insert(presence.user_id, Rc::new(RefCell::new(presence)));
    }

    Ok(presences)
}

pub fn deserialize_private_channels<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<ChannelId, Rc<RefCell<Channel>>>, D::Error> {
    let vec: Vec<Channel> = Deserialize::deserialize(deserializer)?;
    let mut private_channels = HashMap::new();

    for private_channel in vec {
        let id = match private_channel {
            Channel::Group(ref group) => group.borrow().channel_id,
            Channel::Private(ref channel) => channel.borrow().id,
            Channel::Guild(_) => unreachable!("Guild private channel decode"),
            Channel::Category(_) => unreachable!("Channel category private channel decode"),
        };

        private_channels.insert(id, Rc::new(RefCell::new(private_channel)));
    }

    Ok(private_channels)
}

pub fn deserialize_roles<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<RoleId, Rc<RefCell<Role>>>, D::Error> {
    let vec: Vec<Role> = Deserialize::deserialize(deserializer)?;
    let mut roles = HashMap::new();

    for role in vec {
        roles.insert(role.id, Rc::new(RefCell::new(role)));
    }

    Ok(roles)
}

pub fn deserialize_single_recipient<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<Rc<RefCell<User>>, D::Error> {
    let mut users: Vec<User> = Deserialize::deserialize(deserializer)?;
    let user = if users.is_empty() {
        return Err(DeError::custom("Expected a single recipient"));
    } else {
        users.remove(0)
    };

    Ok(Rc::new(RefCell::new(user)))
}

pub fn deserialize_user<'de, D>(deserializer: D)
    -> StdResult<Rc<RefCell<User>>, D::Error> where D: Deserializer<'de> {
    Ok(Rc::new(RefCell::new(User::deserialize(deserializer)?)))
}

pub fn serialize_user<S: Serializer>(
    user: &Rc<RefCell<User>>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    User::serialize(&*user.borrow(), serializer)
}

pub fn deserialize_users<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<UserId, Rc<RefCell<User>>>, D::Error> {
    let vec: Vec<User> = Deserialize::deserialize(deserializer)?;
    let mut users = HashMap::new();

    for user in vec {
        users.insert(user.id, Rc::new(RefCell::new(user)));
    }

    Ok(users)
}

pub fn serialize_users<S: Serializer>(
    users: &HashMap<UserId, Rc<RefCell<User>>>,
    serializer: S
) -> StdResult<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(users.len()))?;

    for user in users.values() {
        seq.serialize_element(&*user.borrow())?;
    }

    seq.end()
}

pub fn deserialize_u16<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<u16, D::Error> {
    deserializer.deserialize_any(U16Visitor)
}

pub fn deserialize_u64<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<u64, D::Error> {
    deserializer.deserialize_any(U64Visitor)
}

pub fn deserialize_voice_states<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<UserId, VoiceState>, D::Error> {
    let vec: Vec<VoiceState> = Deserialize::deserialize(deserializer)?;
    let mut voice_states = HashMap::new();

    for voice_state in vec {
        voice_states.insert(voice_state.user_id, voice_state);
    }

    Ok(voice_states)
}

pub fn serialize_gen_map<K: Eq + Hash, S: Serializer, V: Serialize>(
    map: &HashMap<K, V>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(map.len()))?;

    for value in map.values() {
        seq.serialize_element(&value)?;
    }

    seq.end()
}

pub fn serialize_gen_rc_map<K: Eq + Hash, S: Serializer, V: Serialize>(
    map: &HashMap<K, Rc<RefCell<V>>>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(map.len()))?;

    for value in map.values() {
        if let Ok(item) = value.try_borrow() {
            seq.serialize_element(&*item)?;
        }
    }

    seq.end()
}

#[cfg(all(feature = "cache", feature = "model"))]
pub trait PermissionCheck {
    fn user_has_perms(&self, channel_id: ChannelId, permissions: Permissions)
        -> Result<bool>;
}

#[cfg(all(feature = "cache", feature = "model"))]
impl PermissionCheck for Cache {
    fn user_has_perms(
        &self,
        channel_id: ChannelId,
        mut permissions: Permissions,
    ) -> Result<bool> {
        let current_user = &self.user;

        let channel = match self.channel(channel_id) {
            Some(channel) => channel,
            None => return Err(Error::Model(ModelError::ItemMissing)),
        };

        let guild_id = match channel {
            Channel::Guild(channel) => channel.borrow().guild_id,
            Channel::Group(_) | Channel::Private(_) | Channel::Category(_) => {
                // Both users in DMs, and all users in groups and maybe all channels in categories will
                // have the same
                // permissions.
                //
                // The only exception to this is when the current user is blocked by
                // the recipient in a DM channel, which results in the current user
                // not being able to send messages.
                //
                // Since serenity can't _reasonably_ check and keep track of these,
                // just assume that all permissions are granted and return `true`.
                return Ok(true);
            },
        };

        let guild = match self.guilds.get(&guild_id) {
            Some(guild) => guild,
            None => return Err(Error::Model(ModelError::ItemMissing)),
        };

        let perms = guild
            .borrow()
            .permissions_in(channel_id, current_user.id);

        permissions.remove(perms);

        Ok(permissions.is_empty())
    }
}

macro_rules! ftryopt {
    ($code:expr) => {
        match $code {
            Some(ref v) => v,
            None => return Box::new(::futures::future::err(::Error::Model(
                ::model::ModelError::ClientNotPresent,
            ))),
        }
    };
}

macro_rules! num_visitors {
    ($($visitor:ident: $type:ty),*) => {
        $(
            #[derive(Debug)]
            pub struct $visitor;

            impl<'de> Visitor<'de> for $visitor {
                type Value = $type;

                fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                    formatter.write_str("identifier")
                }

                fn visit_str<E: DeError>(self, v: &str) -> StdResult<Self::Value, E> {
                    v.parse::<$type>().map_err(|_| {
                        let mut s = String::with_capacity(32);
                        s.push_str("Unknown ");
                        s.push_str(stringify!($type));
                        s.push_str(" value: ");
                        s.push_str(v);

                        DeError::custom(s)
                    })
                }

                fn visit_i64<E: DeError>(self, v: i64) -> StdResult<Self::Value, E> { Ok(v as $type) }

                fn visit_u64<E: DeError>(self, v: u64) -> StdResult<Self::Value, E> { Ok(v as $type) }
            }
        )*
    }
}

num_visitors!(U16Visitor: u16, U64Visitor: u64);
