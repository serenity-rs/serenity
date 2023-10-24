use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

use serde::ser::{Serialize, SerializeSeq, Serializer};

use super::prelude::*;

pub fn default_true() -> bool {
    true
}

/// Helper function for `#[serde(skip_serializing_if = "is_false")]`
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_false(v: &bool) -> bool {
    !v
}

#[allow(clippy::unnecessary_wraps)]
pub fn ignore_input<'de, D: Deserializer<'de>>(_: D) -> StdResult<(), D::Error> {
    Ok(())
}

pub fn deserialize_val<T, E>(val: Value) -> StdResult<T, E>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::Error,
{
    T::deserialize(val).map_err(serde::de::Error::custom)
}

pub fn remove_from_map_opt<T, E>(map: &mut JsonMap, key: &str) -> StdResult<Option<T>, E>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::Error,
{
    map.remove(key).map(deserialize_val).transpose()
}

pub fn remove_from_map<T, E>(map: &mut JsonMap, key: &'static str) -> StdResult<T, E>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::Error,
{
    remove_from_map_opt(map, key)?.ok_or_else(|| serde::de::Error::missing_field(key))
}

pub fn add_guild_id_to_map(map: &mut JsonMap, key: &str, id: GuildId) {
    if let Some(array) = map.get_mut(key).and_then(Value::as_array_mut) {
        for value in array {
            if let Some(item) = value.as_object_mut() {
                item.insert("guild_id".to_string(), id.get().into());
            }
        }
    }
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
) -> StdResult<HashMap<ChannelId, GuildChannel>, D::Error> {
    struct TryDeserialize<T>(StdResult<T, String>);
    impl<'de, T: Deserialize<'de>> Deserialize<'de> for TryDeserialize<T> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
            Ok(Self(T::deserialize(deserializer).map_err(|e| e.to_string())))
        }
    }

    let vec: Vec<TryDeserialize<GuildChannel>> = Deserialize::deserialize(deserializer)?;
    let mut map = HashMap::new();

    for channel in vec {
        match channel.0 {
            Ok(channel) => {
                map.insert(channel.id, channel);
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
            url: String::new(),
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
            Channel::Private(channel) => channel.id,
            Channel::Guild(_) => unreachable!("Guild private channel decode"),
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
            mut role,
        } = Event::deserialize(deserializer)?;

        role.guild_id = Some(guild_id);
        Ok(Role::from(role))
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
