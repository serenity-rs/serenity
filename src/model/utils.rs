use std::cell::Cell;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::NonZeroU64;

use arrayvec::ArrayVec;
use serde::de::Error as DeError;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde_cow::CowStr;

use super::prelude::*;
use crate::internal::prelude::*;

pub fn default_true() -> bool {
    true
}

/// Helper function for `#[serde(skip_serializing_if = "is_false")]`
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_false(v: &bool) -> bool {
    !v
}

#[cfg(feature = "model")]
pub(super) fn avatar_url(
    guild_id: Option<GuildId>,
    user_id: UserId,
    hash: Option<&ImageHash>,
) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.is_animated() { "gif" } else { "webp" };

        if let Some(guild_id) = guild_id {
            cdn!("/guilds/{}/users/{}/avatars/{}.{}?size=1024", guild_id, user_id, hash, ext)
        } else {
            cdn!("/avatars/{}/{}.{}?size=1024", user_id, hash, ext)
        }
    })
}

#[cfg(feature = "model")]
pub(super) fn icon_url(id: GuildId, icon: Option<&ImageHash>) -> Option<String> {
    icon.map(|icon| {
        let ext = if icon.is_animated() { "gif" } else { "webp" };

        cdn!("/icons/{}/{}.{}", id, icon, ext)
    })
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

/// Workaround for Discord sending 0 value Ids as default values.
/// This has been fixed properly on next by swapping to a NonMax based impl.
pub fn deserialize_buggy_id<'de, D, Id>(deserializer: D) -> StdResult<Option<Id>, D::Error>
where
    D: Deserializer<'de>,
    Id: From<NonZeroU64>,
{
    if let Some(val) = Option::<StrOrInt<'de>>::deserialize(deserializer)? {
        let val = val.parse().map_err(serde::de::Error::custom)?;
        Ok(NonZeroU64::new(val).map(Id::from))
    } else {
        Ok(None)
    }
}

pub(super) struct SerializeIter<I>(Cell<Option<I>>);

impl<I> SerializeIter<I> {
    pub fn new(iter: I) -> Self {
        Self(Cell::new(Some(iter)))
    }
}

impl<Iter, Item> serde::Serialize for SerializeIter<Iter>
where
    Iter: Iterator<Item = Item>,
    Item: serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(iter) = self.0.take() else {
            return serializer.serialize_seq(Some(0))?.end();
        };

        serializer.collect_seq(iter)
    }
}

pub(super) enum StrOrInt<'de> {
    String(String),
    Str(&'de str),
    Int(u64),
}

impl StrOrInt<'_> {
    pub fn parse(&self) -> Result<u64, std::num::ParseIntError> {
        match self {
            StrOrInt::String(val) => val.parse(),
            StrOrInt::Str(val) => val.parse(),
            StrOrInt::Int(val) => Ok(*val),
        }
    }

    pub fn into_enum<T>(self, string: fn(String) -> T, int: fn(u64) -> T) -> T {
        match self {
            Self::Int(val) => int(val),
            Self::String(val) => string(val),
            Self::Str(val) => string(val.into()),
        }
    }
}

impl<'de> serde::Deserialize<'de> for StrOrInt<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StrOrInt<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string or integer")
            }

            fn visit_borrowed_str<E>(self, val: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StrOrInt::Str(val))
            }

            fn visit_str<E: serde::de::Error>(self, val: &str) -> StdResult<Self::Value, E> {
                self.visit_string(val.into())
            }

            fn visit_string<E: serde::de::Error>(self, val: String) -> StdResult<Self::Value, E> {
                Ok(StrOrInt::String(val))
            }

            fn visit_i64<E: serde::de::Error>(self, val: i64) -> StdResult<Self::Value, E> {
                self.visit_u64(val as _)
            }

            fn visit_u64<E: serde::de::Error>(self, val: u64) -> Result<Self::Value, E> {
                Ok(StrOrInt::Int(val))
            }
        }

        deserializer.deserialize_any(Visitor)
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

/// Used with `#[serde(with = "members")]
pub mod members {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::guild::Member;
    use crate::model::id::UserId;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<UserId, Member>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|member: &Member| member.user.id))
    }

    pub use super::serialize_map_values as serialize;
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
) -> StdResult<FixedArray<ActivityButton>, D::Error> {
    ArrayVec::<_, 2>::deserialize(deserializer).map(|labels| {
        FixedArray::from_vec_trunc(
            labels
                .into_iter()
                .map(|l| ActivityButton {
                    label: l,
                    url: FixedString::default(),
                })
                .collect(),
        )
    })
}

/// Used with `#[serde(with = "roles")]`
pub mod roles {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::SequenceToMapVisitor;
    use crate::model::guild::Role;
    use crate::model::id::RoleId;

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
    use serde_cow::CowStr;

    use crate::internal::prelude::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<FixedArray<FixedString>, D::Error> {
        let str_sequence = CowStr::deserialize(deserializer)?.0;
        let vec = str_sequence.split(", ").map(FixedString::from_str_trunc).collect();

        Ok(FixedArray::from_vec_trunc(vec))
    }

    pub fn serialize<S: Serializer>(
        vec: &FixedArray<FixedString>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let vec: Vec<String> = vec.iter().cloned().map(String::from).collect();
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

pub mod secret {
    use secrecy::{ExposeSecret, Secret, Zeroize};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, S: Deserialize<'de> + Zeroize, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Secret<S>>, D::Error> {
        Option::<S>::deserialize(deserializer).map(|s| s.map(Secret::new))
    }

    pub fn serialize<S: Serialize + Zeroize, Sr: Serializer>(
        secret: &Option<Secret<S>>,
        serializer: Sr,
    ) -> Result<Sr::Ok, Sr::Error> {
        secret.as_ref().map(|s| s.expose_secret()).serialize(serializer)
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

pub fn discord_colours_opt<'de, D>(deserializer: D) -> Result<Option<Vec<Colour>>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec_str: Option<Vec<CowStr<'_>>> = Deserialize::deserialize(deserializer)?;

    let Some(vec_str) = vec_str else { return Ok(None) };

    if vec_str.is_empty() {
        return Ok(None);
    }

    deserialize_colours::<D>(vec_str).map(Some)
}

pub fn discord_colours<'de, D>(deserializer: D) -> Result<Vec<Colour>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec_str: Vec<CowStr<'_>> = Deserialize::deserialize(deserializer)?;

    deserialize_colours::<D>(vec_str)
}

fn deserialize_colours<'de, D>(vec_str: Vec<CowStr<'_>>) -> Result<Vec<Colour>, D::Error>
where
    D: Deserializer<'de>,
{
    vec_str
        .into_iter()
        .map(|s| {
            let s = s.0.strip_prefix('#').ok_or_else(|| DeError::custom("Invalid colour data"))?;

            if s.len() != 6 {
                return Err(DeError::custom("Invalid colour data length"));
            }

            u32::from_str_radix(s, 16)
                .map(Colour::new)
                .map_err(|_| DeError::custom("Invalid colour data"))
        })
        .collect()
}
