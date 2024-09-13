use std::fmt;

use arrayvec::ArrayVec;
use serde::de::Error as DeError;
use serde_cow::CowStr;
use small_fixed_array::FixedString;

use super::prelude::*;

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
pub(super) fn user_banner_url(
    guild_id: Option<GuildId>,
    user_id: UserId,
    hash: Option<&ImageHash>,
) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.is_animated() { "gif" } else { "webp" };

        if let Some(guild_id) = guild_id {
            cdn!("/guilds/{}/users/{}/banners/{}.{}?size=1024", guild_id, user_id, hash, ext)
        } else {
            cdn!("/banners/{}/{}.{}?size=1024", user_id, hash, ext)
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

pub fn remove_from_map<T, E>(map: &mut JsonMap, key: &'static str) -> StdResult<T, E>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::Error,
{
    map.remove(key).ok_or_else(|| serde::de::Error::missing_field(key)).and_then(deserialize_val)
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

    pub fn into_enum<T>(self, string: fn(FixedString) -> T, int: fn(u64) -> T) -> T {
        match self {
            Self::Int(val) => int(val),
            Self::String(val) => string(FixedString::from_string_trunc(val)),
            Self::Str(val) => string(FixedString::from_str_trunc(val)),
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

#[cfg(test)]
#[track_caller]
pub(crate) fn assert_json<T>(data: &T, json: Value)
where
    T: serde::Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    // test serialization
    let serialized = serde_json::to_value(data).unwrap();
    assert!(
        serialized == json,
        "data->JSON serialization failed\nexpected: {json:?}\n     got: {serialized:?}"
    );

    // test deserialization
    let deserialized = serde_json::from_value::<T>(json).unwrap();
    assert!(
        &deserialized == data,
        "JSON->data deserialization failed\nexpected: {data:?}\n     got: {deserialized:?}"
    );
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
        serializer.serialize_str(&join_to_string(", ", vec))
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

    #[allow(clippy::ref_option)]
    pub fn serialize<S: Serialize + Zeroize, Sr: Serializer>(
        secret: &Option<Secret<S>>,
        serializer: Sr,
    ) -> Result<Sr::Ok, Sr::Error> {
        secret.as_ref().map(ExposeSecret::expose_secret).serialize(serializer)
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

// A function used for deserializing components within a MessageUpdateEvent.
// Due to discord now sending the whole message payload, we don't need to distinguish between None
// and empty, as such we always return Some.
pub fn optional_deserialize_components<'de, D>(
    deserializer: D,
) -> Result<Option<FixedArray<ActionRow>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_components(deserializer).map(Some)
}

// Custom deserialize function to deserialize components safely without knocking the whole message
// out when new components are found but not supported.
pub fn deserialize_components<'de, D>(deserializer: D) -> Result<FixedArray<ActionRow>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ComponentsVisitor;

    impl<'de> Visitor<'de> for ComponentsVisitor {
        type Value = FixedArray<ActionRow>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a sequence of ActionRow elements")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut components = Vec::with_capacity(seq.size_hint().unwrap_or_default());

            while let Some(map) = seq.next_element::<JsonMap>()? {
                // We deserialize only the `kind` field to determine the component type.
                // We later use this to check if its a supported component before deserializing the
                // entire payload.
                let raw_kind =
                    map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
                let kind: i64 = deserialize_val(raw_kind)?;

                // Action rows are the only top level component supported in serenity at this time.
                if kind == 1 {
                    let value = Value::from(map);
                    components.push(ActionRow::deserialize(value).map_err(DeError::custom)?);
                } else {
                    // Top level component is not an action row and cannot be supported on
                    // serenity@current without breaking changes, so we skip them.
                    tracing::debug!("Skipping component with unsupported kind: {kind}");
                }
            }

            Ok(FixedArray::from_vec_trunc(components))
        }
    }

    deserializer.deserialize_seq(ComponentsVisitor)
}
