use serde::de::Error as DeError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::*;

#[cfg(feature = "cache")]
use internal::prelude::*;

#[cfg(feature = "cache")]
use super::permissions::Permissions;
#[cfg(feature = "cache")]
use CACHE;

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
        let user_id = member.user.read().unwrap().id;

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
            Channel::Group(ref group) => group.read().unwrap().channel_id,
            Channel::Private(ref channel) => channel.read().unwrap().id,
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

pub fn deserialize_u16<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<u16, D::Error> {
    deserializer.deserialize_u16(U16Visitor)
}

pub fn deserialize_u64<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<u64, D::Error> {
    deserializer.deserialize_u64(U64Visitor)
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

#[cfg(all(feature = "cache", feature = "model"))]
pub fn user_has_perms(channel_id: ChannelId, mut permissions: Permissions) -> Result<bool> {
    let cache = CACHE.read().unwrap();
    let current_user = &cache.user;

    let channel = match cache.channel(channel_id) {
        Some(channel) => channel,
        None => return Err(Error::Model(ModelError::ItemMissing)),
    };

    let guild_id = match channel {
        Channel::Guild(channel) => channel.read().unwrap().guild_id,
        Channel::Group(_) |
        Channel::Private(_) |
        Channel::Category(_) => {
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

    let perms = guild.read().unwrap().permissions_for(
        channel_id,
        current_user.id,
    );

    permissions.remove(perms);

    Ok(permissions.is_empty())
}

pub struct U16Visitor;

impl<'de> Visitor<'de> for U16Visitor {
    type Value = u16;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("identifier")
    }

    fn visit_str<E: DeError>(self, v: &str) -> StdResult<Self::Value, E> {
        match v.parse::<u16>() {
            Ok(v) => Ok(v),
            Err(_) => {
                let mut s = String::new();
                s.push_str("Unknown ");
                s.push_str(stringify!($name));
                s.push_str(" value: ");
                s.push_str(v);

                Err(DeError::custom(s))
            },
        }
    }

    fn visit_i64<E: DeError>(self, v: i64) -> StdResult<Self::Value, E> { Ok(v as u16) }

    fn visit_u64<E: DeError>(self, v: u64) -> StdResult<Self::Value, E> { Ok(v as u16) }
}

pub struct U64Visitor;

impl<'de> Visitor<'de> for U64Visitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("identifier")
    }

    fn visit_str<E: DeError>(self, v: &str) -> StdResult<Self::Value, E> {
        match v.parse::<u64>() {
            Ok(v) => Ok(v),
            Err(_) => {
                let mut s = String::new();
                s.push_str("Unknown ");
                s.push_str(stringify!($name));
                s.push_str(" value: ");
                s.push_str(v);

                Err(DeError::custom(s))
            },
        }
    }

    fn visit_i64<E: DeError>(self, v: i64) -> StdResult<Self::Value, E> { Ok(v as u64) }

    fn visit_u64<E: DeError>(self, v: u64) -> StdResult<Self::Value, E> { Ok(v) }
}
