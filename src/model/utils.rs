use parking_lot::RwLock;
use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::ser::{SerializeSeq, Serialize, Serializer};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc
};
use super::prelude::*;

#[cfg(feature = "cache")]
use internal::prelude::*;

#[cfg(all(feature = "cache", feature = "model"))]
use super::permissions::Permissions;
#[cfg(all(feature = "cache", feature = "model"))]
use CACHE;

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
    -> StdResult<HashMap<ChannelId, Arc<RwLock<GuildChannel>>>, D::Error> {
    let vec: Vec<GuildChannel> = Deserialize::deserialize(deserializer)?;
    let mut map = HashMap::new();

    for channel in vec {
        map.insert(channel.id, Arc::new(RwLock::new(channel)));
    }

    Ok(map)
}

pub fn deserialize_members<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<UserId, Member>, D::Error> {
    let vec: Vec<Member> = Deserialize::deserialize(deserializer)?;
    let mut members = HashMap::new();

    for member in vec {
        let user_id = member.user.read().id;

        members.insert(user_id, member);
    }

    Ok(members)
}

pub fn deserialize_presences<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<UserId, Presence>, D::Error> {
    let vec: Vec<Presence> = Deserialize::deserialize(deserializer)?;
    let mut presences = HashMap::new();

    for presence in vec {
        presences.insert(presence.user_id, presence);
    }

    Ok(presences)
}

pub fn deserialize_private_channels<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<ChannelId, Channel>, D::Error> {
    let vec: Vec<Channel> = Deserialize::deserialize(deserializer)?;
    let mut private_channels = HashMap::new();

    for private_channel in vec {
        let id = match private_channel {
            Channel::Group(ref group) => group.read().channel_id,
            Channel::Private(ref channel) => channel.read().id,
            Channel::Guild(_) => unreachable!("Guild private channel decode"),
            Channel::Category(_) => unreachable!("Channel category private channel decode"),
        };

        private_channels.insert(id, private_channel);
    }

    Ok(private_channels)
}

pub fn deserialize_roles<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<RoleId, Role>, D::Error> {
    let vec: Vec<Role> = Deserialize::deserialize(deserializer)?;
    let mut roles = HashMap::new();

    for role in vec {
        roles.insert(role.id, role);
    }

    Ok(roles)
}

pub fn deserialize_single_recipient<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<Arc<RwLock<User>>, D::Error> {
    let mut users: Vec<User> = Deserialize::deserialize(deserializer)?;
    let user = if users.is_empty() {
        return Err(DeError::custom("Expected a single recipient"));
    } else {
        users.remove(0)
    };

    Ok(Arc::new(RwLock::new(user)))
}

pub fn deserialize_sync_user<'de, D>(deserializer: D)
    -> StdResult<Arc<RwLock<User>>, D::Error> where D: Deserializer<'de> {
    Ok(Arc::new(RwLock::new(User::deserialize(deserializer)?)))
}

pub fn serialize_sync_user<S: Serializer>(
    user: &Arc<RwLock<User>>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    User::serialize(&*user.read(), serializer)
}

pub fn deserialize_users<'de, D: Deserializer<'de>>(
    deserializer: D)
    -> StdResult<HashMap<UserId, Arc<RwLock<User>>>, D::Error> {
    let vec: Vec<User> = Deserialize::deserialize(deserializer)?;
    let mut users = HashMap::new();

    for user in vec {
        users.insert(user.id, Arc::new(RwLock::new(user)));
    }

    Ok(users)
}

pub fn serialize_users<S: Serializer>(
    users: &HashMap<UserId, Arc<RwLock<User>>>,
    serializer: S
) -> StdResult<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(users.len()))?;

    for user in users.values() {
        seq.serialize_element(&*user.read())?;
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

pub fn serialize_gen_locked_map<K: Eq + Hash, S: Serializer, V: Serialize>(
    map: &HashMap<K, Arc<RwLock<V>>>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(map.len()))?;

    for value in map.values() {
        seq.serialize_element(&*value.read())?;
    }

    seq.end()
}

#[cfg(all(feature = "cache", feature = "model"))]
pub fn user_has_perms(channel_id: ChannelId, mut permissions: Permissions) -> Result<bool> {
    let cache = CACHE.read();
    let current_user = &cache.user;

    let channel = match cache.channel(channel_id) {
        Some(channel) => channel,
        None => return Err(Error::Model(ModelError::ItemMissing)),
    };

    let guild_id = match channel {
        Channel::Guild(channel) => channel.read().guild_id,
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

    let guild = match cache.guild(guild_id) {
        Some(guild) => guild,
        None => return Err(Error::Model(ModelError::ItemMissing)),
    };

    let perms = guild
        .read()
        .permissions_in(channel_id, current_user.id);

    permissions.remove(perms);

    Ok(permissions.is_empty())
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

                // This is called when serde_json's `arbitrary_precision` feature is enabled.
                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> StdResult<Self::Value, A::Error> {
                    struct Id {
                        num: $type,
                    }
                    
                    struct StrVisitor;
                    
                    impl<'de> Visitor<'de> for StrVisitor {
                        type Value = $type;
                        
                        fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                            formatter.write_str("string")
                        }
                        
                        fn visit_str<E: DeError>(self, s: &str) -> StdResult<Self::Value, E> { s.parse().map_err(E::custom) }
                        fn visit_string<E: DeError>(self, s: String) -> StdResult<Self::Value, E> { s.parse().map_err(E::custom) }
                    }
                    
                    impl<'de> Deserialize<'de> for Id {
                        fn deserialize<D: Deserializer<'de>>(des: D) -> StdResult<Self, D::Error> {
                            Ok(Id { num: des.deserialize_str(StrVisitor)? })
                        }
                    }
                    
                    map.next_value::<Id>().map(|id| id.num)
                }
            }
        )*
    }
}

num_visitors!(U16Visitor: u16, U64Visitor: u64);
