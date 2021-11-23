use std::marker::PhantomData;
use std::{collections::HashMap, hash::Hash};

use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::ser::{Serialize, SerializeSeq, Serializer};
#[cfg(feature = "simd-json")]
use simd_json::ValueAccess;

#[cfg(all(feature = "cache", feature = "model"))]
use super::permissions::Permissions;
use super::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "cache")]
use crate::internal::prelude::*;
#[cfg(feature = "unstable_discord_api")]
use crate::model::interactions::application_command::*;

pub fn default_true() -> bool {
    true
}

/// Used with `#[serde(with = "emojis")]`
pub mod emojis {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::{guild::Emoji, id::EmojiId};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<EmojiId, Emoji>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|emoji: &Emoji| emoji.id))
    }

    pub use super::serialize_map_values as serialize;
}

pub fn deserialize_guild_channels<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<ChannelId, Channel>, D::Error> {
    struct TryDeserialize<T>(StdResult<T, String>);
    impl<'de, T: Deserialize<'de>> Deserialize<'de> for TryDeserialize<T> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
            Ok(Self(T::deserialize(deserializer).map_err(|e| e.to_string())))
        }
    }

    let vec: Vec<TryDeserialize<Channel>> = Deserialize::deserialize(deserializer)?;
    let mut map = HashMap::new();

    for channel in vec {
        match channel.0 {
            Ok(channel) => {
                map.insert(channel.id(), channel);
            },
            Err(e) => tracing::warn!("skipping guild channel due to deserialization error: {}", e),
        }
    }

    Ok(map)
}

pub fn deserialize_members<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<UserId, Member>, D::Error> {
    deserializer.deserialize_seq(SequenceToMapVisitor::new(|member: &Member| member.user.id))
}

#[cfg(feature = "unstable_discord_api")]
pub fn deserialize_partial_members_map<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<UserId, PartialMember>, D::Error> {
    HashMap::deserialize(deserializer)
}

#[cfg(feature = "unstable_discord_api")]
pub fn deserialize_users<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<UserId, User>, D::Error> {
    HashMap::deserialize(deserializer)
}

#[cfg(feature = "unstable_discord_api")]
pub fn deserialize_roles_map<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<RoleId, Role>, D::Error> {
    HashMap::deserialize(deserializer)
}

#[cfg(feature = "unstable_discord_api")]
pub fn deserialize_channels_map<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<ChannelId, PartialChannel>, D::Error> {
    HashMap::deserialize(deserializer)
}

#[cfg(feature = "unstable_discord_api")]
pub fn deserialize_messages_map<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<MessageId, Message>, D::Error> {
    HashMap::deserialize(deserializer)
}

#[cfg(feature = "unstable_discord_api")]
pub fn deserialize_options<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<Vec<ApplicationCommandInteractionDataOption>, D::Error> {
    Vec::deserialize(deserializer)
}

#[cfg(feature = "unstable_discord_api")]
pub fn deserialize_options_with_resolved<'de, D: Deserializer<'de>>(
    deserializer: D,
    resolved: &ApplicationCommandInteractionDataResolved,
) -> StdResult<Vec<ApplicationCommandInteractionDataOption>, D::Error> {
    let mut options = Vec::deserialize(deserializer)?;

    for option in options.iter_mut() {
        loop_resolved(option, resolved);
    }

    Ok(options)
}

#[cfg(feature = "unstable_discord_api")]
fn try_resolve(
    value: &Value,
    kind: ApplicationCommandOptionType,
    resolved: &ApplicationCommandInteractionDataResolved,
) -> Option<ApplicationCommandInteractionDataOptionValue> {
    let string = value.as_str();

    match kind {
        ApplicationCommandOptionType::User => {
            let id = &UserId(string?.parse().ok()?);

            let user = resolved.users.get(id)?.to_owned();
            let member = resolved.members.get(id).map(ToOwned::to_owned);

            Some(ApplicationCommandInteractionDataOptionValue::User(user, member))
        },
        ApplicationCommandOptionType::Role => {
            let id = &RoleId(string?.parse().ok()?);

            let role = resolved.roles.get(id)?.to_owned();

            Some(ApplicationCommandInteractionDataOptionValue::Role(role))
        },
        ApplicationCommandOptionType::Channel => {
            let id = &ChannelId(string?.parse().ok()?);

            let channel = resolved.channels.get(id)?.to_owned();

            Some(ApplicationCommandInteractionDataOptionValue::Channel(channel))
        },
        ApplicationCommandOptionType::Mentionable => {
            let id: u64 = string?.parse().ok()?;

            if let Some(user) = resolved.users.get(&UserId(id)) {
                let user = user.to_owned();
                let member = resolved.members.get(&UserId(id)).map(|m| m.to_owned());

                Some(ApplicationCommandInteractionDataOptionValue::User(user, member))
            } else {
                let role = resolved.roles.get(&RoleId(id))?.to_owned();

                Some(ApplicationCommandInteractionDataOptionValue::Role(role))
            }
        },
        ApplicationCommandOptionType::String => {
            Some(ApplicationCommandInteractionDataOptionValue::String(string?.to_owned()))
        },
        ApplicationCommandOptionType::Integer => {
            Some(ApplicationCommandInteractionDataOptionValue::Integer(value.as_i64()?))
        },
        ApplicationCommandOptionType::Boolean => {
            Some(ApplicationCommandInteractionDataOptionValue::Boolean(value.as_bool()?))
        },
        ApplicationCommandOptionType::Number => {
            Some(ApplicationCommandInteractionDataOptionValue::Number(value.as_f64()?))
        },
        _ => None,
    }
}

#[cfg(feature = "unstable_discord_api")]
fn loop_resolved(
    options: &mut ApplicationCommandInteractionDataOption,
    resolved: &ApplicationCommandInteractionDataResolved,
) {
    if let Some(ref value) = options.value {
        options.resolved = try_resolve(value, options.kind, resolved);
    }

    for option in options.options.iter_mut() {
        loop_resolved(option, resolved);
    }
}

/// Used with `#[serde(with = "presences")]`
pub mod presences {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::{gateway::Presence, id::UserId};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<UserId, Presence>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|p: &Presence| p.user.id))
    }

    pub use super::serialize_map_values as serialize;
}

pub fn deserialize_buttons<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<Vec<ActivityButton>, D::Error> {
    let labels = Vec::deserialize(deserializer)?;
    let mut buttons = vec![];

    for label in labels {
        buttons.push(ActivityButton {
            label,
            url: "".to_owned(),
        });
    }

    Ok(buttons)
}

/// Used with `#[serde(with = "private_channels")]`
pub mod private_channels {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::{channel::Channel, id::ChannelId};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<ChannelId, Channel>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|channel: &Channel| match channel {
            Channel::Private(ref channel) => channel.id,
            Channel::Guild(_) => unreachable!("Guild private channel decode"),
            Channel::Category(_) => unreachable!("Channel category private channel decode"),
        }))
    }

    pub use super::serialize_map_values as serialize;
}

/// Used with `#[serde(with = "roles")]`
pub mod roles {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::{guild::Role, id::RoleId};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<RoleId, Role>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|role: &Role| role.id))
    }

    pub use super::serialize_map_values as serialize;
}

/// Used with `#[serde(with = "stickers")]`
pub mod stickers {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::{id::StickerId, sticker::Sticker};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<StickerId, Sticker>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|sticker: &Sticker| sticker.id))
    }

    pub use super::serialize_map_values as serialize;
}

/// Used with `#[serde(with = "comma_separated_string")]`
pub mod comma_separated_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<String>, D::Error> {
        let str_sequence = String::deserialize(deserializer)?;
        let vec = str_sequence.split(", ").map(str::to_owned).collect();

        Ok(vec)
    }

    #[allow(clippy::ptr_arg)]
    pub fn serialize<S: Serializer>(vec: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&vec.join(", "))
    }
}

/// Used with `#[serde(with = "single_recipient")]`
pub mod single_recipient {
    use serde::{de::Error, ser::SerializeSeq, Deserialize, Deserializer, Serializer};

    use crate::model::user::User;

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<User, D::Error> {
        let mut users: Vec<User> = Vec::deserialize(deserializer)?;

        let user = if users.is_empty() {
            return Err(Error::custom("Expected a single recipient"));
        } else {
            users.remove(0)
        };

        Ok(user)
    }

    pub fn serialize<S: Serializer>(user: &User, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(1))?;

        seq.serialize_element(user)?;

        seq.end()
    }
}

pub fn deserialize_u16<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<u16, D::Error> {
    deserializer.deserialize_any(U16Visitor)
}

pub fn deserialize_opt_u16<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<Option<u16>, D::Error> {
    deserializer.deserialize_option(OptU16Visitor)
}

pub fn deserialize_u64<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<u64, D::Error> {
    deserializer.deserialize_any(U64Visitor)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_u64<S: Serializer>(data: &u64, ser: S) -> StdResult<S::Ok, S::Error> {
    ser.serialize_str(&data.to_string())
}

pub fn deserialize_voice_states<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<HashMap<UserId, VoiceState>, D::Error> {
    deserializer.deserialize_seq(SequenceToMapVisitor::new(|state: &VoiceState| state.user_id))
}

pub fn serialize_map_values<K, S: Serializer, V: Serialize>(
    map: &HashMap<K, V>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(map.len()))?;

    for value in map.values() {
        seq.serialize_element(&value)?;
    }

    seq.end()
}

/// Tries to find a user's permissions using the cache.
/// Unlike [`user_has_perms`], this function will return `true` even when
/// the permissions are not in the cache.
#[cfg(all(feature = "cache", feature = "model"))]
#[inline]
pub fn user_has_perms_cache(
    cache: impl AsRef<Cache>,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
    permissions: Permissions,
) -> Result<()> {
    if match user_has_perms(cache, channel_id, guild_id, permissions) {
        Err(Error::Model(err)) => err.is_cache_err(),
        result => result?,
    } {
        Ok(())
    } else {
        Err(Error::Model(ModelError::InvalidPermissions(permissions)))
    }
}

#[cfg(all(feature = "cache", feature = "model"))]
pub fn user_has_perms(
    cache: impl AsRef<Cache>,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
    mut permissions: Permissions,
) -> Result<bool> {
    let cache = cache.as_ref();

    let channel = match cache.channel(channel_id) {
        Some(channel) => channel,
        None => return Err(Error::Model(ModelError::ChannelNotFound)),
    };

    // Both users in DMs, all users in groups, and maybe all channels in categories
    // will have the same permissions.
    //
    // The only exception to this is when the current user is blocked by
    // the recipient in a DM channel, preventing the current user
    // from sending messages.
    //
    // Since serenity can't _reasonably_ check and keep track of these,
    // just assume that all permissions are granted and return `true`.
    let (guild_id, guild_channel) = match channel {
        Channel::Guild(channel) => (channel.guild_id, channel),
        Channel::Category(_) => return Ok(true),
        Channel::Private(_) => match guild_id {
            Some(_) => return Err(Error::Model(ModelError::InvalidChannelType)),
            None => return Ok(true),
        },
    };

    let guild = match cache.guild(guild_id) {
        Some(guild) => guild,
        None => return Err(Error::Model(ModelError::GuildNotFound)),
    };

    let member = match guild.members.get(&cache.current_user().id) {
        Some(member) => member,
        None => return Err(Error::Model(ModelError::MemberNotFound)),
    };

    let perms = guild.user_permissions_in(&guild_channel, member)?;

    permissions.remove(perms);

    Ok(permissions.is_empty())
}

/// Deserializes a sequence and builds a `HashMap` with the key extraction function.
struct SequenceToMapVisitor<F, V> {
    key: F,
    marker: PhantomData<V>,
}

impl<F, V> SequenceToMapVisitor<F, V> {
    fn new(key: F) -> Self {
        Self {
            key,
            marker: PhantomData,
        }
    }
}

impl<'de, F, K, V> Visitor<'de> for SequenceToMapVisitor<F, V>
where
    K: Eq + Hash,
    V: Deserialize<'de>,
    F: FnMut(&V) -> K,
{
    type Value = HashMap<K, V>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("sequence")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> StdResult<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut map = seq.size_hint().map_or_else(HashMap::new, HashMap::with_capacity);
        while let Some(elem) = seq.next_element()? {
            map.insert((self.key)(&elem), elem);
        }

        Ok(map)
    }
}

macro_rules! num_visitors {
    ($($visitor:ident: $type:ty),*) => {
        $(
            #[derive(Debug)]
            pub struct $visitor;

            impl<'de> Visitor<'de> for $visitor {
                type Value = $type;

                fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                    formatter.write_str("number")
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

macro_rules! num_opt_visitors {
    ($($visitor:ident: $type:ty, $visitor_impl:ident),*) => {
        $(
            #[derive(Debug)]
            pub struct $visitor;

            impl<'de> Visitor<'de> for $visitor {
                type Value = Option<$type>;

                fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                    formatter.write_str("optional number")
                }

                fn visit_unit<E: DeError>(self) -> StdResult<Self::Value, E> {
                    Ok(None)
                }

                fn visit_none<E: DeError>(self) -> StdResult<Self::Value, E> {
                    Ok(None)
                }

                fn visit_some<D>(self, deserializer: D) -> StdResult<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_any($visitor_impl).map(Some)
                }
            }
        )*
    }
}

num_opt_visitors!(OptU16Visitor: u16, U16Visitor);
