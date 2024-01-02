/// Used with `#[serde(with = "users")]`
pub mod users {
    use std::collections::HashMap;

    use serde::Deserializer;

    use crate::model::id::UserId;
    use crate::model::user::User;
    use crate::model::utils::SequenceToMapVisitor;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<UserId, User>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|u: &User| u.id))
    }

    pub use crate::model::utils::serialize_map_values as serialize;
}

/// Used with `#[serde(with = "webhooks")]`
pub mod webhooks {
    use std::collections::HashMap;

    use serde::Deserializer;

    use crate::model::id::WebhookId;
    use crate::model::utils::SequenceToMapVisitor;
    use crate::model::webhook::Webhook;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<WebhookId, Webhook>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|h: &Webhook| h.id))
    }

    pub use crate::model::utils::serialize_map_values as serialize;
}

/// Deserializes an optional string containing a valid integer as `Option<u64>`.
///
/// Used with `#[serde(with = "optional_string")]`.
pub mod optional_string {
    use std::fmt;
    use std::marker::PhantomData;
    use std::str::FromStr;

    use serde::de::{Deserializer, Error, Visitor};
    use serde::ser::Serializer;

    // Workaround for https://github.com/LPGhatguy/nonmax/issues/17
    pub(crate) trait TryFromU64
    where
        Self: Sized,
    {
        type Err: fmt::Display;
        fn try_from_u64(value: u64) -> Result<Self, Self::Err>;
    }

    impl TryFromU64 for u64 {
        type Err = std::convert::Infallible;
        fn try_from_u64(value: u64) -> Result<Self, Self::Err> {
            Ok(value)
        }
    }

    impl TryFromU64 for nonmax::NonMaxU64 {
        type Err = nonmax::TryFromIntError;
        fn try_from_u64(value: u64) -> Result<Self, Self::Err> {
            Self::try_from(value)
        }
    }

    impl TryFromU64 for nonmax::NonMaxU32 {
        type Err = nonmax::TryFromIntError;
        fn try_from_u64(value: u64) -> Result<Self, Self::Err> {
            Self::try_from(u32::try_from(value)?)
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr + TryFromU64,
        <T as FromStr>::Err: fmt::Display,
    {
        deserializer.deserialize_option(OptionalStringVisitor::<T>(PhantomData))
    }

    #[allow(clippy::ref_option)]
    pub fn serialize<S: Serializer>(
        value: &Option<impl ToString>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.serialize_some(&value.to_string()),
            None => serializer.serialize_none(),
        }
    }

    struct OptionalStringVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for OptionalStringVisitor<T>
    where
        T: FromStr + TryFromU64,
        <T as FromStr>::Err: fmt::Display,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an optional integer or a string with a valid number inside")
        }

        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_any(OptionalStringVisitor(PhantomData))
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        /// Called by the `simd_json` crate
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_u64<E: Error>(self, val: u64) -> Result<Self::Value, E> {
            T::try_from_u64(val).map(Some).map_err(Error::custom)
        }

        fn visit_str<E: Error>(self, string: &str) -> Result<Self::Value, E> {
            string.parse().map(Some).map_err(Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::optional_string;
    use crate::json::{assert_json, json};

    #[test]
    fn optional_string_module() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct T {
            #[serde(with = "optional_string")]
            opt: Option<u64>,
        }

        let value = T {
            opt: Some(12345),
        };

        assert_json(&value, json!({"opt": "12345"}));
    }
}
