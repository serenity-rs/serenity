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

/// Deserializes an optional string containing a valid integer as ´Option<u64>`.
///
/// Used with `#[serde(with = "optional_string")]`.
pub mod optional_string {
    use std::fmt;

    use serde::de::{Deserializer, Error, Visitor};
    use serde::ser::Serializer;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<u64>, D::Error> {
        deserializer.deserialize_option(OptionalStringVisitor)
    }

    pub fn serialize<S: Serializer>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.serialize_some(&value.to_string()),
            None => serializer.serialize_none(),
        }
    }

    struct OptionalStringVisitor;

    impl<'de> Visitor<'de> for OptionalStringVisitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an optional integer or a string with a valid number inside")
        }

        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_any(OptionalStringVisitor)
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        /// Called by the `simd_json` crate
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_u64<E: Error>(self, val: u64) -> Result<Option<u64>, E> {
            Ok(Some(val))
        }

        fn visit_str<E: Error>(self, string: &str) -> Result<Option<u64>, E> {
            string.parse().map(Some).map_err(Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_test::Token;

    use super::optional_string;

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

        serde_test::assert_tokens(&value, &[
            Token::Struct {
                name: "T",
                len: 1,
            },
            Token::Str("opt"),
            Token::Some,
            Token::Str("12345"),
            Token::StructEnd,
        ]);

        serde_test::assert_de_tokens(&value, &[
            Token::Struct {
                name: "T",
                len: 1,
            },
            Token::Str("opt"),
            Token::Str("12345"),
            Token::StructEnd,
        ]);
    }
}
