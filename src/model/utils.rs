use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

use serde::ser::{Serialize, SerializeSeq, Serializer};

#[cfg(all(feature = "cache", feature = "model"))]
use super::permissions::Permissions;
use super::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "cache")]
use crate::internal::prelude::*;
use crate::model::application::command::CommandOptionType;
use crate::model::application::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
    CommandDataResolved,
};

pub fn default_true() -> bool {
    true
}

/// Helper function for `#[serde(skip_serializing_if = "is_false")]`
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_false(v: &bool) -> bool {
    !v
}

/// Used with `#[serde(with = "emojis")]`
pub mod emojis {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::guild::Emoji;
    use crate::model::id::EmojiId;

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

pub fn deserialize_options_with_resolved<'de, D: Deserializer<'de>>(
    deserializer: D,
    resolved: &CommandDataResolved,
) -> StdResult<Vec<CommandDataOption>, D::Error> {
    let mut options = Vec::deserialize(deserializer)?;

    for option in &mut options {
        loop_resolved(option, resolved);
    }

    Ok(options)
}

fn try_resolve(
    value: &Value,
    kind: CommandOptionType,
    resolved: &CommandDataResolved,
) -> Option<CommandDataOptionValue> {
    let string = value.as_str();

    match kind {
        CommandOptionType::User => {
            let id = &UserId(string?.parse().ok()?);

            let user = resolved.users.get(id)?.clone();
            let member = resolved.members.get(id).cloned();

            Some(CommandDataOptionValue::User(user, member))
        },
        CommandOptionType::Role => {
            let id = &RoleId(string?.parse().ok()?);

            let role = resolved.roles.get(id)?.clone();

            Some(CommandDataOptionValue::Role(role))
        },
        CommandOptionType::Channel => {
            let id = &ChannelId(string?.parse().ok()?);

            let channel = resolved.channels.get(id)?.clone();

            Some(CommandDataOptionValue::Channel(channel))
        },
        CommandOptionType::Mentionable => {
            let id: u64 = string?.parse().ok()?;

            if let Some(user) = resolved.users.get(&UserId(id)) {
                let user = user.clone();
                let member = resolved.members.get(&UserId(id)).cloned();

                Some(CommandDataOptionValue::User(user, member))
            } else {
                let role = resolved.roles.get(&RoleId(id))?.clone();

                Some(CommandDataOptionValue::Role(role))
            }
        },
        CommandOptionType::String => Some(CommandDataOptionValue::String(string?.to_owned())),
        CommandOptionType::Integer => Some(CommandDataOptionValue::Integer(value.as_i64()?)),
        CommandOptionType::Boolean => Some(CommandDataOptionValue::Boolean(value.as_bool()?)),
        CommandOptionType::Number => Some(CommandDataOptionValue::Number(value.as_f64()?)),
        CommandOptionType::Attachment => {
            let id = &AttachmentId(string?.parse().ok()?);

            let attachment = resolved.attachments.get(id)?.clone();

            Some(CommandDataOptionValue::Attachment(attachment))
        },
        _ => None,
    }
}

fn loop_resolved(options: &mut CommandDataOption, resolved: &CommandDataResolved) {
    if let Some(ref value) = options.value {
        options.resolved = try_resolve(value, options.kind, resolved);
    }

    for option in &mut options.options {
        loop_resolved(option, resolved);
    }
}

/// Used with `#[serde(with = "presences")]`
pub mod presences {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::gateway::Presence;
    use crate::model::id::UserId;

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
    use crate::model::channel::Channel;
    use crate::model::id::ChannelId;

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

    use serde::{Deserialize, Deserializer};

    use super::SequenceToMapVisitor;
    use crate::model::guild::{InterimRole, Role};
    use crate::model::id::{GuildId, RoleId};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<RoleId, Role>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|role: &Role| role.id))
    }

    pub use super::serialize_map_values as serialize;

    /// Helper to deserialize `GuildRoleCreateEvent` and `GuildRoleUpdateEvent`.
    pub fn deserialize_event<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Role, D::Error> {
        #[derive(Deserialize)]
        struct Event {
            guild_id: GuildId,
            role: InterimRole,
        }

        let Event {
            guild_id,
            role,
        } = Event::deserialize(deserializer)?;

        let mut role = Role::from(role);
        role.guild_id = guild_id;

        Ok(role)
    }
}

/// Used with `#[serde(with = "stickers")]`
pub mod stickers {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::id::StickerId;
    use crate::model::sticker::Sticker;

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
    use serde::de::Error;
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serializer};

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
pub(in crate::model) struct SequenceToMapVisitor<F, V> {
    key: F,
    marker: PhantomData<V>,
}

impl<F, V> SequenceToMapVisitor<F, V> {
    pub fn new(key: F) -> Self {
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

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
